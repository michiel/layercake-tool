# Tips and snippets

## File watcher examples

#### Example linux using inotifywait

```bash
while true; \
  do inotifywait -e close-write out/kvm-control-flow.dot && \
  dot -Tpng out/kvm-control-flow.dot -o out/kvm-control-flow.png; \
done
```

#### Example MacOS using built-in stat

```bash
prev_mod_time=$(stat -f "%m" out/kvm-control-flow.dot)                                                                                                                                                                                                                                                                                                [0/1306]

while true; do                                                                         
  sleep 1                                                                              
  new_mod_time=$(stat -f "%m" out/kvm-control-flow.dot)
  if [ "$new_mod_time" -ne "$prev_mod_time" ]; then
    dot -Tpng out/kvm-control-flow.dot -o out/kvm-control-flow.png
    prev_mod_time=$new_mod_time                                                                                                                                                                                                                                                                                                                               
  fi                                                                                                                                                                                                                                                                                                                                                          
done
```

#### Example MacOS using fswatch

```bash
while true; do
  fswatch -1 -e ".*" -i "out/kvm-control-flow.dot" out/ && \
  dot -Tpng out/kvm-control-flow.dot -o out/kvm-control-flow.png
done
```


