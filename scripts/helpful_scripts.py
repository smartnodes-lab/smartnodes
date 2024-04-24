from brownie import SmartNodes, accounts, config


def get_account(index=None):
    # Use a specific account from the brownie accounts
    if index:
        return accounts[index]
    # Use accounts loaded from `brownie accounts generate`
    if config["wallets"]["from_key"]:
        return accounts.add(config["wallets"]["from_key"])
    # Fallback to the first local ganache account
    return accounts[0]
