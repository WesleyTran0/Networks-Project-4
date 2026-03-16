# High-Level Approach

Firstly, after recognizing that the arguments for this program is much simpler (and more difficulties
with encodings later), I decided to not use any packages like json serialization and clap for argument parsing.
To start, I took the python files and converted it to the best of my ability to idiomatic rust. In the process,
I added simple things that would need to be added later anyway like a window and a buffer to hold on to packets
I received out of order. This allowed me to pass the first couple levels of tests and from there, I went through
all the tests and implemented all the features until the tests passed.

# Challenges

Implementing all the features to pass the tests in the first place were their own challenge. I originally tried to
sent data as json, but after experience loss converting data to strings, I decided against it. My next best idea
was to encode the data as base64, but after realizing that base64 would result in 25% overhead, I changed the way
I was structuring my packets and eventually just stored the data as bits/bytes.

Optimizations were their struggle on their own. Upon first attempt, I implemented TCP Reno strategies, but was still
not getting full score. From there, I reduced my timeout time from 100ms to 1ms and started processing all acks before
looking to fill up the sender window again.

# Tests

I did not test very much outside of the provided tests, but after completing `run` functions for send and realizing the
scope that a single function was handling, I split `run` into 3 functions and let `run` loop those 3 functions until
the data was all sent.
