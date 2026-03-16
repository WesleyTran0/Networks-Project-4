#!/usr/bin/env -S python3 -u

import argparse
import json
import math
import select
import socket
import struct
import sys
import time

DATA_SIZE = 1375


class Sender:
    def __init__(self, host, port):
        self.host = host
        self.port = int(port)
        self.log("Sender starting up using port %s" % self.port)
        self.socket = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
        self.socket.bind(("0.0.0.0", 0))
        self.waiting = False

        self.remote_host = None
        self.remote_port = None

    def log(self, message):
        sys.stderr.write(message + "\n")
        sys.stderr.flush()

    def send(self, message):
        self.log("Sending message '%s'" % json.dumps(message))
        self.socket.sendto(json.dumps(message).encode("utf-8"), (self.host, self.port))

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
            self.log("Received message %s" % data)
            return json.loads(data.decode("utf-8"))

    def run(self):
        seq = 0
        while True:
            sockets = [self.socket, sys.stdin] if not self.waiting else [self.socket]

            socks = select.select(sockets, [], [], 0.1)[0]
            for conn in socks:
                if conn == self.socket:
                    data = self.recv(conn)

                    if data:
                        self.waiting = False
                elif conn == sys.stdin:
                    data = sys.stdin.read(DATA_SIZE)
                    if len(data) == 0:
                        self.log("All done!")
                        sys.exit(0)

                    msg = {"type": "msg", "data": data, "seq": seq}
                    self.send(msg)
                    self.waiting = True
                    seq += 1

        return


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="send data")
    parser.add_argument("host", type=str, help="Remote host to connect to")
    parser.add_argument("port", type=int, help="UDP port number to connect to")
    args = parser.parse_args()
    sender = Sender(args.host, args.port)
    sender.run()
