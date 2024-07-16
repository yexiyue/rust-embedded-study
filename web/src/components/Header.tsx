import { GithubOutlined, UnorderedListOutlined } from "@ant-design/icons";
import { Button, Drawer } from "antd";
import { useState } from "react";
import { NavLink } from "react-router-dom";

export const Header = () => {
  const [open, setOpen] = useState(false);
  return (
    <header className=" h-8 md:h-12  flex justify-center items-center bg-slate-100 relative">
      <Button
        icon={<UnorderedListOutlined />}
        type="text"
        className="absolute left-4 md:hidden"
        onClick={() => setOpen(true)}
      />
      <p className="md:absolute md:left-4 md:text-xl">Rust Embedded Study</p>
      <div className="hidden md:flex md:gap-4 md:text-2xl">
        <NavLink
          to="/http"
          className={({ isActive }) => (isActive ? " text-blue-500" : "")}
        >
          http
        </NavLink>
        <NavLink
          to="/ws"
          className={({ isActive }) => (isActive ? " text-blue-500" : "")}
        >
          ws
        </NavLink>
        <NavLink
          to="/mqtt"
          className={({ isActive }) => (isActive ? " text-blue-500" : "")}
        >
          mqtt
        </NavLink>
      </div>
      <Button
        icon={<GithubOutlined />}
        className="absolute right-4"
        type="text"
        href="https://github.com/yexiyue/rust-embedded-study"
        target="_blank"
      ></Button>
      <Drawer
        closable={false}
        onClose={() => setOpen(false)}
        open={open}
        placement="left"
        width={180}
      >
        <div className="flex flex-col gap-2">
          <NavLink
            to="/http"
            className={({ isActive }) =>
              ` px-2 py-1 rounded ${
                isActive
                  ? "text-green md:text-blue-500 bg-gray-100 "
                  : "bg-gray-200"
              } `
            }
          >
            http
          </NavLink>
          <NavLink
            to="/ws"
            className={({ isActive }) =>
              ` px-2 py-1 rounded ${
                isActive
                  ? "text-green md:text-blue-500 bg-gray-100 "
                  : "bg-gray-200"
              } `
            }
          >
            ws
          </NavLink>
          <NavLink
            to="/mqtt"
            className={({ isActive }) =>
              ` px-2 py-1 rounded ${
                isActive
                  ? "text-green md:text-blue-500 bg-gray-100 "
                  : "bg-gray-200"
              } `
            }
          >
            mqtt
          </NavLink>
        </div>
      </Drawer>
    </header>
  );
};
