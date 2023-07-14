import base64
import threading
import socket
import time


# Represents a connection between two nodes,
class UserBridge(threading.Thread):
    def __init__(self, host: str, port: int, sock: socket.socket):
        super(UserBridge, self).__init__()

        self.host = host
        self.port = port
        self.sock = sock

        self.terminate_flag = threading.Event()
        self.sock.settimeout(5.0)

    def compress(self, data):
        # Pickle and compress model?
        pass

    def decompress(self, data):
        pass

    def send(self, data):
        pass

    def stop(self):
        self.terminate_flag.set()

    def run(self):
        buffer = b""

        while not self.terminate_flag.is_set():
            chunk = b""

            try:
                chunk = self.sock.recv(4096)

            except socket.timeout:
                pass

            except Exception as e:
                self.terminate_flag.set()

            if chunk != b"":
                buffer += chunk
            else:
                break

        if buffer != b"":
            pass

        self.sock.settimeout(None)
        self.sock.close()
