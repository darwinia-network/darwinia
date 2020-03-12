package main

import (
	"bytes"
	"crypto/sha256"
	"encoding/csv"
	"encoding/json"
	"errors"
	"fmt"
	"github.com/shopspring/decimal"
	"io"
	"io/ioutil"
	"math/big"
	"math/rand"
	"net/http"
	"strings"
	"sync"
)

const (
	trongrid = "https://api.trongrid.io/wallet/getaccount"
	infura   = "https://mainnet.infura.io/v3/"
)

type (
	TronAccount struct {
		Type string `json:"type,omitempty"`
	}

	Web3Res struct {
		Result string `json:"result"`
	}

	Result struct {
		Eth  map[string]decimal.Decimal `json:"eth"`
		Tron map[string]decimal.Decimal `json:"tron"`
		Dot  map[string]decimal.Decimal `json:"dot"`
	}
)

func main() {
	var addresses []string
	result := Result{
		Eth:  make(map[string]decimal.Decimal),
		Tron: make(map[string]decimal.Decimal),
		Dot:  make(map[string]decimal.Decimal),
	}

	var read = func(filePath string, target map[string]decimal.Decimal, k func(string) string, f func(string) decimal.Decimal) {
		b, err := ioutil.ReadFile(filePath)
		if err != nil {
			panic(err)
		}
		r := csv.NewReader(strings.NewReader(string(b[:])))
		for {
			record, err := r.Read()
			if err == io.EOF {
				break
			}
			if err != nil {
				fmt.Println(err)
			}
			addresses = append(addresses, k(record[0]))
			target[k(record[0])] = target[k(record[0])].Add(f(record[1]))
		}
	}

	// Bank
	read("data/bank.csv",
		result.Eth,
		func(s string) string { return s },
		func(f string) decimal.Decimal { return decimal.RequireFromString(f).Mul(decimal.New(1, 18)) })

	read("data/bank_tron.csv",
		result.Tron,
		func(s string) string { return s },
		func(f string) decimal.Decimal { return decimal.RequireFromString(f).Mul(decimal.New(1, 18)) })

	// Eth
	read("data/eth.csv",
		result.Eth,
		func(s string) string { return s },
		func(d string) decimal.Decimal {
			var balance decimal.Decimal
			if b, err := decimal.NewFromString(d); err == nil {
				balance = b
			} else {
				if sr := strings.Split(d, "E"); len(sr) > 2 {
					balance = decimal.RequireFromString(sr[0]).Mul(decimal.RequireFromString(sr[1]))
				}
			}
			return balance.Mul(decimal.New(1, 18))
		})

	// Tron
	read("data/tron.csv",
		result.Tron,
		func(s string) string { return TrxBase58toHexAddress(s) },
		func(f string) decimal.Decimal { return decimal.RequireFromString(f) })

	// Dot
	read("data/dot.csv",
		result.Dot,
		func(s string) string { return s },
		func(f string) decimal.Decimal { return decimal.RequireFromString(f).Mul(decimal.New(50, 18)) })

	// filter
	var contracts []string

	var delAddress = func(address string) {
		fmt.Println(address, "is contract ")
		contracts = append(contracts, address)
	}

	var checkContract = func(addr string) {
		if strings.HasPrefix(addr, "41") {
			var tr TronAccount
			b, err := postWithJson([]byte(fmt.Sprintf(`{"address":"%s"}`, addr)), trongrid)
			if err != nil {
				fmt.Println(err)
			}
			if _ = json.Unmarshal(b, &tr); tr.Type == "Contract" {
				delAddress(addr)
			}
		} else {
			b, err := postWithJson([]byte(fmt.Sprintf(`{"jsonrpc":"2.0","method":"eth_getCode","params":["%s","latest"],"id":%d}`, addr, rand.Intn(1000))), infura)
			if err != nil {
				fmt.Println(err)
			}
			var w Web3Res
			if _ = json.Unmarshal(b, &w); w.Result != "0x" {
				delAddress(addr)
			}
		}
	}

	var wg sync.WaitGroup
	var jobsChan = make(chan string, 5)
	for i := 0; i < 5; i++ {
		go func() {
			for addr := range jobsChan {
				checkContract(addr)
				wg.Done()
			}
		}()
	}

	for _, address := range addresses {
		jobsChan <- address
		wg.Add(1)

	}
	wg.Wait()

	var filter = func(r map[string]decimal.Decimal) {
		for address := range r {
			if StringInSlice(address, contracts) {
				delete(r, address)
			}
		}
	}
	filter(result.Eth)
	filter(result.Tron)
	filter(result.Dot)

	b, _ := json.Marshal(result)
	fmt.Println(string(b))
}

func StringInSlice(a string, list []string) bool {
	for _, b := range list {
		if b == a {
			return true
		}
	}
	return false
}

func postWithJson(data []byte, url string) ([]byte, error) {
	client := &http.Client{}
	req, _ := http.NewRequest("POST", url, bytes.NewBuffer(data))
	req.Header.Set("Content-Type", "application/json")
	resp, err := client.Do(req)
	if err != nil {
		return nil, err
	}
	if resp.Body == nil {
		return nil, nil
	}
	defer resp.Body.Close()
	body, _ := ioutil.ReadAll(resp.Body)
	return body, nil
}

const (
	alphabetIdx0 = '1'
)

var b58 = [256]byte{
	255, 255, 255, 255, 255, 255, 255, 255,
	255, 255, 255, 255, 255, 255, 255, 255,
	255, 255, 255, 255, 255, 255, 255, 255,
	255, 255, 255, 255, 255, 255, 255, 255,
	255, 255, 255, 255, 255, 255, 255, 255,
	255, 255, 255, 255, 255, 255, 255, 255,
	255, 0, 1, 2, 3, 4, 5, 6,
	7, 8, 255, 255, 255, 255, 255, 255,
	255, 9, 10, 11, 12, 13, 14, 15,
	16, 255, 17, 18, 19, 20, 21, 255,
	22, 23, 24, 25, 26, 27, 28, 29,
	30, 31, 32, 255, 255, 255, 255, 255,
	255, 33, 34, 35, 36, 37, 38, 39,
	40, 41, 42, 43, 255, 44, 45, 46,
	47, 48, 49, 50, 51, 52, 53, 54,
	55, 56, 57, 255, 255, 255, 255, 255,
	255, 255, 255, 255, 255, 255, 255, 255,
	255, 255, 255, 255, 255, 255, 255, 255,
	255, 255, 255, 255, 255, 255, 255, 255,
	255, 255, 255, 255, 255, 255, 255, 255,
	255, 255, 255, 255, 255, 255, 255, 255,
	255, 255, 255, 255, 255, 255, 255, 255,
	255, 255, 255, 255, 255, 255, 255, 255,
	255, 255, 255, 255, 255, 255, 255, 255,
	255, 255, 255, 255, 255, 255, 255, 255,
	255, 255, 255, 255, 255, 255, 255, 255,
	255, 255, 255, 255, 255, 255, 255, 255,
	255, 255, 255, 255, 255, 255, 255, 255,
	255, 255, 255, 255, 255, 255, 255, 255,
	255, 255, 255, 255, 255, 255, 255, 255,
	255, 255, 255, 255, 255, 255, 255, 255,
	255, 255, 255, 255, 255, 255, 255, 255,
}
var bigRadix = big.NewInt(58)

// Decode decodes a modified base58 string to a byte slice.
func Base58Decode(b string) []byte {
	answer := big.NewInt(0)
	j := big.NewInt(1)

	scratch := new(big.Int)
	for i := len(b) - 1; i >= 0; i-- {
		tmp := b58[b[i]]
		if tmp == 255 {
			return []byte("")
		}
		scratch.SetInt64(int64(tmp))
		scratch.Mul(j, scratch)
		answer.Add(answer, scratch)
		j.Mul(j, bigRadix)
	}

	tmpval := answer.Bytes()

	var numZeros int
	for numZeros = 0; numZeros < len(b); numZeros++ {
		if b[numZeros] != alphabetIdx0 {
			break
		}
	}
	flen := numZeros + len(tmpval)
	val := make([]byte, flen)
	copy(val[numZeros:], tmpval)

	return val
}

// ErrChecksum indicates that the checksum of a check-encoded string does not verify against
// the checksum.
var Base58ErrChecksum = errors.New("checksum error")

// ErrInvalidFormat indicates that the check-encoded string has an invalid format.
var Base58ErrInvalidFormat = errors.New("invalid format: version and/or checksum bytes missing")

// checksum: first four bytes of sha256^2
func checksum(input []byte) (cksum [4]byte) {
	h := sha256.Sum256(input)
	h2 := sha256.Sum256(h[:])
	copy(cksum[:], h2[:4])
	return
}

// Base58CheckDecode decodes a string that was encoded with CheckEncode and verifies the checksum.
func Base58CheckDecode(input string) (result []byte, version byte, err error) {
	decoded := Base58Decode(input)
	if len(decoded) < 5 {
		return nil, 0, Base58ErrInvalidFormat
	}
	version = decoded[0]
	var cksum [4]byte
	copy(cksum[:], decoded[len(decoded)-4:])
	if checksum(decoded[:len(decoded)-4]) != cksum {
		return nil, 0, Base58ErrChecksum
	}
	payload := decoded[1 : len(decoded)-4]
	result = append(result, payload...)
	return
}

func TrxBase58toHexAddress(base58 string) string {
	last20b, version, err := Base58CheckDecode(base58)

	if err != nil {
		return ""
	}

	var buf bytes.Buffer
	buf.WriteByte(version)
	buf.Write(last20b)

	return fmt.Sprintf("%x", buf.Bytes())
}
