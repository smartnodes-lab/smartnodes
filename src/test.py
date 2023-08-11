from p2p.node import Node
import time


node1 = Node("127.0.0.1", 5025, debug=True)
node2 = Node("127.0.0.1", 5026, debug=True)

node1.start()
node2.start()

node2.connect_with_node("127.0.0.1", 5025)

time.sleep(0.01)

node2.send_to_nodes(b"data-transfer-test")

time.sleep(1)

node1.debug_print(node1.all_nodes)
node2.debug_print(node2.all_nodes)

node1.stop()
node2.stop()
