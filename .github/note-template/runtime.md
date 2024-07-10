## {{ .Env.CHAIN | strings.Title }}
<h3 align="right">Upgrade Priority HIGH :red_circle:</h3>
<h3 align="center">!!All the nodes must be upgraded due to the new asynchronous backing feature!!</h4>

#### Whitelist Hash
```
{{ .Env.WHITELIST_HASH }}
```
#### Blake2 256 Hash
```
{{ (ds "srtool").runtimes.compressed.subwasm.blake2_256 }}
```
