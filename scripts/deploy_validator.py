from brownie import accounts, config, network, Contract, SmartNodes, TransparentProxy, ProxyAdmin
from .helpful_scripts import get_account, encode_function_data, upgrade
from dotenv import load_dotenv
import json

load_dotenv("..")

def main():
    admin_account = accounts[0]
    proxy_admin = ProxyAdmin.deploy({"from": admin_account})

    print(f"Deploying to {network.show_active()}")
    smartnodes = SmartNodes.deploy({"from": admin_account})

    encoded_init_function = encode_function_data(initializer=smartnodes.initialize)

    proxy = TransparentProxy.deploy(
        smartnodes.address,
        proxy_admin.address,
        encoded_init_function,
        {"from": admin_account}
    )
    print(f"Proxy deployed to {proxy}")

    proxy_smartnodes = Contract.from_abi("SmartNodes", proxy.address, SmartNodes.abi)

    proxy_smartnodes.initialize({"from": accounts[0]})

    proxy_smartnodes.mintTokens({"from": accounts[1]})
    proxy_smartnodes.approve(accounts[1], 100 ** 18, {"from": accounts[1]})
    proxy_smartnodes.createValidator("asd", {"from": accounts[1]}) 

    print(proxy_smartnodes.emissionRate({"from": accounts[1]}))
    proxy_smartnodes.mintTokens({"from": accounts[1]})
    proxy_smartnodes.mintTokens({"from": accounts[1]})
    proxy_smartnodes.mintTokens({"from": accounts[1]})
    print(proxy_smartnodes.emissionRate({"from": accounts[1]}))
    print(proxy_smartnodes.validators(1))

    # proxy_smartnodes.createValidator(
    #     "21c99fa3c263570d20132c24ef1b347e1b8afcdcfe88c303fb1f45b84b387a5b",
    #     {"from": accounts[0]}
    # )
    # validator_contract.createValidator(
    #     "dd24f40e6f597f435fcda42465ce0ee8e59ca4ee06ec7681bb251885bb959b3d",
    #     {"from": accounts[1]}
    # )