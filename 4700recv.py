#!/usr/bin/env -S python3 -u

import argparse
import json
import math
import select
import socket
import struct
import sys
import time


class Receiver:
    def __init__(self):
        self.socket = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        self.socket.bind(("0.0.0.0", 0))
        self.port = self.socket.getsockname()[1]
        self.log("Bound to port %d" % self.port)

        self.remote_host = None
        self.remote_port = None

    def send(self, message):
        self.log("Sent message %s" % json.dumps(message))
        self.socket.sendto(
            json.dumps(message).encode("utf-8"), (self.remote_host, self.remote_port)
        )

    def recv(self, socket):
        data, addr = socket.recvfrom(65535)

        # Grab the remote host/port if we don't already have it
        if self.remote_host is None:
            self.remote_host = addr[0]
            self.remote_port = addr[1]

        # Make sure we're talking to the same remote host
        if addr != (self.remote_host, self.remote_port):
            self.log("Error:  Received response from unexpected remote; ignoring")
            return None
        else:
            self.log("Received message %s" % data.decode("utf-8"))
            return json.loads(data.decode("utf-8"))

    def log(self, message):
        sys.stderr.write(message + "\n")
        sys.stderr.flush()

    def run(self):
        while True:
            socks = select.select([self.socket], [], [])[0]
            for conn in socks:
                msg = self.recv(conn)

                if msg:
                    # Print out the data to stdout
                    print(msg["data"], end="", flush=True)

                    # Always send back an ack
                    self.send({"type": "ack", "seq": msg["seq"]})

        return


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="receive data")
    args = parser.parse_args()
    sender = Receiver()
    sender.run()
