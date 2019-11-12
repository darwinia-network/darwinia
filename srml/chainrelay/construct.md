- Bridge
    - EOS bridge
    - Ethereum bridge
        - Deposit Pool (shared ?)
        - Deposit Value (adjustable)
        - Verified Header (Vec\<Header> or MPT\<Header>)
        - Unverified (HashMap\<PrevHash, Header> ?)
        - ...
    - ...
- Relayer
    - ...

---

- submit_header(relayer, header)
- lock()
- redeem(account, transaction)
- challenge(relayer) ?  
- punish(relayer) ?
- reward(relayer) ?
- verify_submit(header)
    1. if exists?
    2. verify (difficulty + prev_hash + nonce)
    3. challenge
- verify_lock(transaction)
    1. get release value
    2. verify most-worked
    3. ...
- release(account, value)
