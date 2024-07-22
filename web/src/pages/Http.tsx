import { App, Button, ColorPicker, Space } from "antd";
import { Color } from "antd/es/color-picker";
import { useState } from "react";

export const Http = () => {
  const [color, setColor] = useState<Color>();

  const { message } = App.useApp();
  return (
    <div className="w-full h-full p-4 flex justify-center">
      <div className="md:w-[400px] w-[200px] flex flex-col gap-2">
        <p className="text-[18px]">设置颜色值</p>
        <div>
          <ColorPicker
            defaultValue="#00ff00"
            value={color}
            onChange={(value) => {
              setColor(value);
            }}
            disabledAlpha
          />
        </div>
        <div className="mt-6">
          <Space>
            <Button
              type="primary"
              onClick={async () => {
                try {
                  const res = await fetch(`/set-color`, {
                    method: "post",
                    headers: {
                      "Context-Type": "application/json",
                    },
                    body: JSON.stringify({
                      color: color
                        ? {
                            r: color.toRgb().r,
                            g: color.toRgb().g,
                            b: color.toRgb().b,
                          }
                        : { r: 0, g: 255, b: 0 },
                    }),
                  });
                  if (res.status === 200) {
                    message.success("设置成功");
                  }
                } catch (error) {
                  message.error("设置失败");
                }
              }}
            >
              调整灯光
            </Button>
            <Button
              type="primary"
              onClick={async () => {
                try {
                  const res = await fetch(`/shutdown`);
                  if (res.status === 200) {
                    message.success("关灯成功");
                  }
                } catch (error) {
                  message.error("关灯失败");
                }
              }}
            >
              关灯
            </Button>
          </Space>
        </div>
      </div>
    </div>
  );
};
