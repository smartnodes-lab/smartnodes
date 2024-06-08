from brownie import accounts, config, network, Contract, TransparentProxy, ProxyAdmin, SmartnodesMultiSig
from .helpful_scripts import get_account, encode_function_data, upgrade
from dotenv import load_dotenv
import json

load_dotenv("..")


def main():
    # Account to deploy the proxy (proxy admin, to become a DAO of sorts)
    account = accounts[0]
    proxy_admin = ProxyAdmin.deploy({"from": account})

    # Deploy Smartnodes
    print(f"Deploying to {network.show_active()}")
    smartnodes = SmartnodesMultiSig.deploy({"from": account})
    encoded_init_function = encode_function_data(initializer=smartnodes.initialize)
    proxy = TransparentProxy.deploy(
        smartnodes.address,
        proxy_admin.address,
        encoded_init_function,
        {"from": account}
    )
    print(f"Smartnodes Proxy Deployed to {proxy}")
    sno = Contract.from_abi("Smartnodes", proxy.address, SmartnodesMultiSig.abi)
    sno.initialize({"from": accounts[1]})
