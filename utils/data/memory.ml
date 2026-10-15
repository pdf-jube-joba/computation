let read ~default array addr =
  if addr < 0 then failwith "Negative address"
  else if addr < Array.length array then array.(addr)
  else default

let write ~default array addr v =
  if addr < 0 || addr >= Sys.max_array_length then
    failwith "Memory address out of range";
  let out = Array.make (max (Array.length array) (addr + 1)) default in
  Array.blit array 0 out 0 (Array.length array);
  out.(addr) <- v;
  out
