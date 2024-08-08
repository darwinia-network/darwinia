## {{ .Env.CHAIN | strings.Title }}
<h3 align="right">Upgrade Priority HIGH :red_circle:</h3>
<h4 align="center">!!All nodes need to be updated to this(`v6.6.4`) version prior to the upcoming `v6.6.5` release because of the new asynchronous backing feature!!</h4>

#### Whitelist Hash
```
{{ .Env.WHITELIST_HASH }}
```
#### Blake2 256 Hash
```
{{ (ds "srtool").runtimes.compressed.subwasm.blake2_256 }}
```
