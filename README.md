# NEAR Mappings

## Overview

Protocol to maintain mappings between NEAR account and various other blockchain accounts, ip addresses and an

## Protocol

Maintains map of `<account_id, label> -> <content>`

Contract supports delegating updates to another account via `<account_id> -> <account_id>`

Protocol has next methods:
- `set(account_id: Option<AccountId>, label: String, content: Option<String>)` - for given account sets content for label. Account id is either empty or must have delegate to the caller.
- `get(account_id: AccountId, label: String) -> String` - returns stored content for given account and label.
- `delegate(account_id: Option<AccountId>)` - allows to delegate control of the caller to given account.

## Standards

| Label | Description | Format |
| - | - | - |
| evm | List of EVM addresses | string[] |
| ens | List of ENS addresses | string[] |
| aaaa | AAAA DNS record | string |
| cname | CNAME DNS record | string |
