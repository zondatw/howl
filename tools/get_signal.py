#!/usr/bin/env python
import signal
import sys

fd = open("./xxx.log", "w+")

def signal_handler(signum, frame):
    print("Signal Number:", signum, " Frame: ", frame)
    fd.write(f"Signal Number: {signum}, Frame: {frame}")
    fd.close()

valid_signals = signal.valid_signals()
# print(valid_signals)
for _signal in list(valid_signals):
    try:
        signal.signal(_signal, signal_handler)
    except Exception as err:
        print(f"Register signal failed: {_signal} -> {err}")
        pass
print('Press Ctrl+C')
signal.pause()
