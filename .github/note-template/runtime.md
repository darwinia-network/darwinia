## {{ .Env.CHAIN | strings.Title }}
<h4 align="right">Upgrade Priority LOW :green_circle:</h4>

#### Whitelist Hash
```
{{ .Env.WHITELIST_HASH }}
```
#### Blake2 256 Hash
```
{{ (ds "srtool").runtimes.compressed.subwasm.blake2_256 }}
```
