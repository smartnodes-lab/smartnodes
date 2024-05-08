from brownie import accounts, config, network, Contract, ValidatorMultiSig
from .helpful_scripts import get_account, encode_function_data, upgrade
from dotenv import load_dotenv
import json

load_dotenv("..")

def main():
    validator_contract = ValidatorMultiSig.deploy({"from": accounts[0]})
    
    validator_contract.