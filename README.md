# Fox-Monitor

This is a simple application built using the [Foxglove SDK](https://github.com/foxglove/foxglove-sdk/tree/main) that logs live system information and outputs it to an MCAP file or Foxglove itself.

## Usage

Run the following command to log your system information

```shell
cargo run -- --cpu --memory --system --overwrite
```

## Options

<table>
  <thead>
    <tr>
      <th>Description</th>
      <th>Short form</th>
      <th>Long form</th>
    </tr>
  </thead>
  <tbody>
      <tr>
        <td>Log cpu info</td>
        <td>-c</td>
        <td>--cpu</td>
      </tr>
      <tr>
        <td>Log memory info</td>
        <td>-m</td>
        <td>--memory</td>
      </tr>
      <tr>
        <td>Log components temperature</td>
        <td>-t</td>
        <td>--temperature</td>
      </tr>
      <tr>
        <td>Log disks info</td>
        <td>-d</td>
        <td>--disks</td>
      </tr>
      <tr>
        <td>Log networks info</td>
        <td>-n</td>
        <td>--networks</td>
      </tr>
      <tr>
        <td>Log processes info</td>
        <td>-p</td>
        <td>--processes</td>
      </tr>
      <tr>
        <td>Log system info</td>
        <td>-s</td>
        <td>--system</td>
      </tr>
      <tr>
        <td>Interval between logs in seconds [default: 1]</td>
        <td>-i <INTERVAL></td>
        <td>--interval <INTERVAL></td>
      </tr>
      <tr>
        <td>If provided, the program will exit after the timeout (in seconds)</td>
        <td></td>
        <td>--timeout <TIMEOUT></td>
      </tr>
      <tr>
        <td>Output format (mcap file, websocket server, or both) [default: both] [possible values: mcap, websocket, both]</td>
        <td>-f</td>
        <td>--format <FORMAT></td>
      </tr>
      <tr>
        <td>Output path for mcap file [default: output.mcap]</td>
        <td></td>
        <td>--path <PATH></td>
      </tr>
      <tr>
        <td>If set, overwrite an existing mcap file</td>
        <td>-o</td>
        <td>--overwrite</td>
      </tr>
      <tr>
        <td>Print help</td>
        <td>-h</td>
        <td>--help</td>
      </tr>
      <tr>
        <td>Print version</td>
        <td>-v</td>
        <td>--version</td>
      </tr>
  </tbody>
</table>