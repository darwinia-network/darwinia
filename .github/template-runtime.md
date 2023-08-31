## {{ .Env.CHAIN | strings.Title }}
<h4 align="right">Upgrade Priority LOW :green_circle:</h4>

#### Proposal Hash
```
{{ (ds "srtool").runtimes.compressed.subwasm.proposal_hash }}
```
#### Blake2 256 Hash
```
{{ (ds "srtool").runtimes.compressed.subwasm.blake2_256 }}
```
