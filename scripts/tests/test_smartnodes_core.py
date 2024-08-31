from brownie import accounts, config, network, Contract, SmartnodesCore, TransparentProxy, ProxyAdmin, SmartnodesMultiSig
from scripts.helpful_scripts import get_account, encode_function_data, upgrade
from eth_abi import encode
from dotenv import load_dotenv, set_key
from web3 import Web3
import hashlib
import os

load_dotenv("..")

def deploy_smartnodes(account):
    sno = SmartnodesCore.deploy({"from": account})
    encoded_init_function = encode_function_data(initializer=sno.initialize)
    return sno


def main():
    account = accounts[0]
    sno = deploy_smartnodes(account)
    sno.createUser(hashlib.sha256().hexdigest(), {"from": account})
    sno.requestJob(
        hashlib.sha256().hexdigest(),
        hashlib.sha256(b"123").hexdigest(),
        [1, 1],
        {"from": account}
    )