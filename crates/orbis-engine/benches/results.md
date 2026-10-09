``` 
cargo bench --bench hdr_bench --quiet -- conversion

conversion: image_rs ser
                        time:   [59.056 ms 60.156 ms 61.306 ms]
conversion: image_rs par
                        time:   [49.255 ms 49.773 ms 50.312 ms]
conversion: stb_image flat_map_iter + collect
                        time:   [45.103 ms 45.668 ms 46.273 ms]
conversion: stb_image par_chunks_mut + zip
                        time:   [39.540 ms 40.103 ms 40.715 ms]
```

``` 
cargo bench --bench hdr_bench --quiet -- load 

load image_rs           time:   [47.125 ms 48.262 ms 49.449 ms]
load stb_image          time:   [38.111 ms 39.063 ms 40.079 ms]

```


