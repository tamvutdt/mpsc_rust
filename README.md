# MPSC Rust
Implement mpsc in rust based on multiple spsc configuration.
SPSCQueue implement based on https://github.com/rigtorp/SPSCQueue with a little modification with batch recv.
A little benchmark has been made and the result quite impressive:

- On Intel Core i5 10400F: SPSC: ~500M msg/s, MPSC (6 pub): ~480M msg/s
- On M2 8 core (4 performance core): SPSC: ~200M msg/s, MPSC (4 pub): ~800M msg/s


