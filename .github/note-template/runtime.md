## {{ .Env.RUNTIME | strings.Title }}
<h4 align="right">Upgrade Priority LOW :green_circle:</h3>

### Digest
```json
{{ (ds "prr" | data.ToJSONPretty "  ") }}
```

### Whitelist Hash
```txt
{{ .Env.WHITELIST_HASH }}
```
