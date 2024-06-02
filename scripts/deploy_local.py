from brownie import accounts, MockVRFCoordinator, network, Contract, SmartNodes, TransparentProxy, ProxyAdmin, SmartNodesMultiSig
from .helpful_scripts import get_account, encode_function_data, upgrade
from dotenv import load_dotenv
import json
import time

load_dotenv("..")

def main():
    # Deploy mocks
    admin_account = accounts[0]
    mock_vrf_coordinator = MockVRFCoordinator.deploy(100000000000000000, 1000000000, {"from": admin_account})

    # Account to deploy the proxy (proxy admin, to become a DAO of sorts)
    proxy_admin = ProxyAdmin.deploy({"from": admin_account})
    print(f"Deploying to {network.show_active()}")

    # Deploy smartnodes iteration v1
    smartnodes = SmartNodes.deploy({"from": admin_account})

    # point transparent proxy to smartnodes
    encoded_init_function = encode_function_data(initializer=smartnodes.initialize)
    proxy = TransparentProxy.deploy(
        smartnodes.address,
        proxy_admin.address,
        encoded_init_function,
        {"from": admin_account}
    )
    print(f"Proxy deployed to {proxy}")

    validator_contract = SmartNodesMultiSig.deploy(proxy.address, {"from": admin_account})

    proxy_smartnodes = Contract.from_abi("SmartNodes", proxy.address, SmartNodes.abi)
    
    tx = mock_vrf_coordinator.createSubscription({"from": admin_account})
    sub_id = tx.events["SubscriptionCreated"][0]["subId"]

    mock_vrf_coordinator.fundSubscription(sub_id, 10e18)

    proxy_smartnodes.initialize(validator_contract.address, mock_vrf_coordinator.address, sub_id, {"from": admin_account})

    mock_vrf_coordinator.addConsumer(sub_id, proxy_smartnodes.address)

    proxy_smartnodes.mintTokens(10e18, {"from": accounts[1]})
    proxy_smartnodes.mintTokens(10e18, {"from": accounts[2]})
    proxy_smartnodes.approve(accounts[1], 10e18, {"from": accounts[1]})
    proxy_smartnodes.approve(accounts[2], 10e18, {"from": accounts[2]})

    proxy_smartnodes.createValidator(
        "21c99fa3c263570d20132c24ef1b347e1b8afcdcfe88c303fb1f45b84b387a5b",
        {"from": accounts[1]}
    )
    proxy_smartnodes.createValidator(
        "dd24f40e6f597f435fcda42465ce0ee8e59ca4ee06ec7681bb251885bb959b3d",
        {"from": accounts[2]}
    )

    # # proxy_smartnodes.createValidator(
    # #     "bd9e075b31d32220361afe2e0bf4e863700d25189a7f406847aae82ef39429ea",
    # #     {"from": accounts[2]}
    # # )

    proxy_smartnodes.createUser(
        "0d976b7e1fd59537000313e274dc6a9d035ebaf95f4b8857740f7c799abd8629",
        {"from": accounts[3]}
    )
    
    # proxy_smartnodes.requestJob(
    #     1,
    #     100_000_000,
    #     {"from": accounts[3]}
    # )

    # print("-------CONTRACT STATE--------")
    # print(f"Validator #1: {proxy_smartnodes.validators(1)}")
    # print(f"Validator #2: {proxy_smartnodes.validators(2)}")
    # print(f"User: {proxy_smartnodes.users(1)}")
    # print(f"Job Request: {proxy_smartnodes.jobs(1)}")
    # time.sleep(3)
    # print(f"Job Request: {proxy_smartnodes.jobs(1)}")

    # selected_validators = proxy_smartnodes.getJobRequestValidators(1, {"from": accounts[2]}) 
    # for acc in selected_validators:
    #     proxy_smartnodes.approveJobCreation(1, {"from": acc})

    # print(f"JobRequest: {proxy_smartnodes.jobRequests(1)}")
    # print(f"Job: {proxy_smartnodes.jobs(1)}")
    # print(f"User: {proxy_smartnodes.users(1)}")
