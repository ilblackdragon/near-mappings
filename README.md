# NEAR Mappings

## Overview

Protocol to maintain mappings between NEAR account and various other blockchain accounts, ip addresses and other future use cases.

## Protocol

Maintains map of `<account_id, label> -> <content>`

Contract supports delegating updates to another account via `<account_id> -> <account_id>`

Contract supports validation of values for some specific values. See section below for detailed description.

Protocol has next methods:

| Function | Description |
| - | - |
| `set(id: Option<Id>, label: String, content: Option<String>, proof: Option<String>)` | For given account sets content for this label. Account id is either empty or must have delegate to the caller. `proof` is provided when a |
| `get(id: Id, label: String, validated: Option<bool>) -> Option<String>` | Returns stored content for given account and label. If `validated` is true and value was not validated, will return `ERR_NOT_VALIDATED` |
| `delegate(account_id: Option<AccountId>)` | Allows to delegate control of the caller to given account. | 

## Standards

`Id` can take next options:
- `{AccountId: account_id}`
- `{EvmAddress: address}`, where `address` is hex encoded string in the format of `0x...`

| Label | Description | Format |
| - | - | - |
| near | List of NEAR account ids | string[] |
| evm | List of EVM addresses | string[] |
| ens | List of ENS addresses | string[] |
| aaaa | AAAA DNS record | string |
| cname | CNAME DNS record | string |
