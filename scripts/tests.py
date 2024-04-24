from brownie import SmartNodes, accounts, config
from scripts.helpful_scripts import get_account


smartnodes = SmartNodes.deploy({"from": accounts[0]})


def test_create_user(ind):
    account = get_account(ind)
    initial_user_count = smartnodes.userIdCounter()
    smartnodes.createUser({"from": account})
    assert smartnodes.userIdCounter() == initial_user_count + 1


def test_create_validator(ind, id):
    account = get_account(ind)
    smartnodes.createValidator(id, {"from": account})
    assert smartnodes.validatorIdCounter() > 1  # Assuming starts at 1


def test_lock_and_unlock_tokens():
    account = get_account()
    smartnodes.createUser({"from": account})
    smartnodes.createValidator("HASH1234", {"from": account})
    initial_balance = smartnodes.balanceOf(account)
    lock_amount = 100
    smartnodes.lockTokens(lock_amount, {"from": account})
    assert smartnodes.validators(1).locked == lock_amount
    # Assuming some time manipulation or block number increase here if necessary
    smartnodes.unlockTokens(lock_amount, {"from": account})
    assert smartnodes.balanceOf(account) == initial_balance


def main():
    test_create_validator(0, "val1")
    test_create_validator(1, "val2")
    test_create_validator(2, "val3")
    test_create_user(3)
