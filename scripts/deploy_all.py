from brownie import accounts, config, network, Contract, SmartnodesCore, TransparentProxy, ProxyAdmin, SmartnodesMultiSig
from scripts.helpful_scripts import get_account, encode_function_data, upgrade
from eth_abi import encode
from dotenv import load_dotenv, set_key
import json
import os

load_dotenv("..")


def main():
    smartnodes_address = os.getenv("SMARTNODES_ADDRESS")
    smartnodes_multisig_address = os.getenv("SMARTNODES_MULTISIG_ADDRESS")

    # Account to deploy the proxy (proxy admin, to become a DAO of sorts)
    account = accounts[0]

    if not smartnodes_address:
        proxy_admin = ProxyAdmin.deploy({"from": account})

        # Deploy Smartnodes
        print(f"Deploying to {network.show_active()}")
        smartnodes = SmartnodesCore.deploy({"from": account})
        encoded_init_function = encode_function_data(initializer=smartnodes.initialize)
        proxy = TransparentProxy.deploy(
            smartnodes.address,
            proxy_admin.address,
            encoded_init_function,
            {"from": account}
        )
        print(f"Smartnodes Proxy Deployed to {proxy}")
        set_key(".env", "SMARTNODES_ADDRESS", proxy.address)

        sno = Contract.from_abi("Smartnodes", proxy.address, SmartnodesCore.abi)
        sno.initialize({"from": account})

        # Deploy Smartnodes validator multisig
        print(f"Deploying to {network.show_active()}")
        smartnodes_multisig = SmartnodesMultiSig.deploy({"from": account})
        encoded_init_function = encode_function_data(initializer=smartnodes_multisig.initialize)
        proxy = TransparentProxy.deploy(
            smartnodes_multisig.address,
            proxy_admin.address,
            encoded_init_function,
            {"from": account}
        )
        print(f"SnoMultiSig Proxy Deployed to {proxy}")

        set_key(".env", "SMARTNODES_MULTISIG_ADDRESS", proxy.address)

        sno_multisig = Contract.from_abi("SnoMultiSig", proxy.address, SmartnodesMultiSig.abi)
        sno_multisig.initialize(sno.address, {"from": account})
        sno.setValidatorContract(sno_multisig, {"from": account})

        # sno = Contract("0xfA9C7f3f463CD4f32F9fd7596F5701997f6474b7")
        # proxy_admin.upgrade(sno, )
        
        sno.mintTokens(account, 100_000e18, {"from": account})
        sno.approve(account, 100_000e18, {"from": account})
        sno.createValidator(
            "21c99fa3c263570d20132c24ef1b347e1b8afcdcfe88c303fb1f45b84b387a5b",
            {"from": account}
        )
        sno_multisig.addValidator(account, {"from": account})

        # sno.mintTokens(10e18, {"from": accounts[2]})
        # sno.approve(accounts[2], 10e18, {"from": accounts[2]})
        # sno.createValidator(
        #     "dd24f40e6f597f435fcda42465ce0ee8e59ca4ee06ec7681bb251885bb959b3d",
        #     {"from": accounts[2]}
        # )

        # sno.createValidator(
        #     "bd9e075b31d32220361afe2e0bf4e863700d25189a7f406847aae82ef39429ea",
        #     {"from": accounts[3]}
        # )

        sno.createUser(
            "0d976b7e1fd59537000313e274dc6a9d035ebaf95f4b8857740f7c799abd8629",
            {"from": accounts[4]}
        )
    else:
        sno = Contract.from_abi("Smartnodes", smartnodes_address, SmartnodesCore.abi)
        sno_multisig = Contract.from_abi("Smartnodes", smartnodes_multisig_address, SmartnodesMultiSig.abi)
    
    sno.requestJob(
        [13e8],
        {"from": accounts[4]}
    )

    print(f"Validator: {sno.validators(1)}")
    print(f"User: {sno.users(1)}")
    print(f"Job: {sno.jobs(1)}")
    
    function_types = [2]
    job_id = 1
    worker_addresses = [accounts[3].address]
    
    data = encode(
        ["uint256", "address[]"],
        [job_id, worker_addresses]
    )

    sno_multisig.createProposal(function_types, [data], {"from": account})
    tx = sno_multisig.approveTransaction(0, {"from": account})

    print(tx.events)
    print(f"Validator: {sno.validators(1)}")
    print(f"User: {sno.users(1)}")
    print(f"Job: {sno.jobs(1)}")
