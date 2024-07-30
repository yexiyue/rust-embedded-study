import { Button } from "antd";

export const Bt = () => {
  return (
    <div>
      bt
      <Button onClick={test}>点击连接蓝牙</Button>
    </div>
  );
};

async function test() {
  //@ts-ignore
  let res = await navigator.bluetooth.requestDevice({
    filters: [{ name: "ESP32" }],
    optionalServices: [0x8848],
  });
  await res.gatt.connect();
  let services = await res.gatt.getPrimaryServices();
  console.log(services);
}
