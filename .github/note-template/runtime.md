## {{ .Env.CHAIN | strings.Title }}
<h3 align="right">Upgrade Priority LOW :green_circle:</h3>

#### Whitelist Hash
```
{{ .Env.WHITELIST_HASH }}
```
#### Blake2 256 Hash
```
{{ (ds "srtool").runtimes.compressed.subwasm.blake2_256 }}
```
