import { Outlet } from "react-router";
import { Header } from "./Header";

export const Layout = () => {
  return (
    <div className="h-screen w-screen">
      <Header />
      <Outlet />
    </div>
  );
};
