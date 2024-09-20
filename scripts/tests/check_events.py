from brownie import accounts, config, network, Contract, SmartnodesCore, TransparentProxy, ProxyAdmin, SmartnodesMultiSig
from scripts.helpful_scripts import get_account, encode_function_data, upgrade
from eth_abi import encode
from dotenv import load_dotenv, set_key
from web3 import Web3
import hashlib
import json
import time
import os

load_dotenv(".env", override=True)


def deploy_proxy_admin(account):
    proxy_address = os.getenv("SMARTNODES_ADMIN_ADDRESS")

    if proxy_address:
        proxy_admin = Contract.from_abi("SmartnodesProxyAdmin", proxy_address, ProxyAdmin.abi)
    else:
        raise "Proxy Admin contract not found!"
    
    return proxy_admin


def deploy_smartnodes(account, proxy_admin):
    smartnodes_address = os.getenv("SMARTNODES_ADDRESS")

    if smartnodes_address:
        sno_proxy = Contract.from_abi("SmartnodesCore", smartnodes_address, SmartnodesCore.abi)
    
    else:
        raise "SmartnodesCore contract not found!"
    
    return sno_proxy


def deploy_smartnodesValidator(account, proxy_admin):
    smartnodes_multisig_address = os.getenv("SMARTNODES_MULTISIG_ADDRESS")

    if smartnodes_multisig_address:
        sno_multisig_proxy = Contract.from_abi("SmartnodesMultiSig", smartnodes_multisig_address, SmartnodesMultiSig.abi)
    else:
        raise "SmartnodesMultiSig contract not found!"

    return sno_multisig_proxy


def initialize_contracts(account, core, multisig):
    core.initialize(accounts[:3], account, {'from': account})
    multisig.initialize(core.address, {"from": account})
    core.setValidatorContract(multisig, {"from": account})
    


def main():
    # Account to deploy the proxy (proxy admin, to become a DAO of sorts)
    account = accounts[0]

    proxy_admin = deploy_proxy_admin(account)
    sno = deploy_smartnodes(account, proxy_admin)
    sno_multisig = deploy_smartnodesValidator(account, proxy_admin)
    initialize_contracts(account, sno, sno_multisig)

    print()
