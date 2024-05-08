from brownie import SmartNodes, accounts, config
import eth_utils


def get_account(index=None):
    # Use a specific account from the brownie accounts
    if index:
        return accounts[index]
    # Use accounts loaded from `brownie accounts generate`
    if config["wallets"]["from_key"]:
        return accounts.add(config["wallets"]["from_key"])
    # Fallback to the first local ganache account
    return accounts[0]


def encode_function_data(initializer=None, *args):
    if len(args) == 0 or not initializer:
        return eth_utils.to_bytes(hexstr="0x")  # we send blank hex data
    return initializer.encode_input(*args)


def upgrade(
    account,
    proxy,
    newimplementation_address,
    proxy_admin_contract=None,
    initializer=None,
    *args
):
    transaction = None
    if proxy_admin_contract:
        if initializer:
            encoded_function_call = encode_function_data(initializer, *args)
            transaction = proxy_admin_contract.upgradeAndCall(
                proxy.address,
                newimplementation_address,
                encoded_function_call,
                {"from": account},
            )
        else:
            transaction = proxy_admin_contract.upgrade(
                proxy.address, newimplementation_address, {"from": account}
            )
    else:
        if initializer:
            encoded_function_call = encode_function_data(initializer, *args)
            transaction = proxy.upgradeToAndCall(
                newimplementation_address, encoded_function_call, {"from": account}
            )
        else:
            transaction = proxy.upgradeTo(newimplementation_address, {"from": account})
    return transaction
