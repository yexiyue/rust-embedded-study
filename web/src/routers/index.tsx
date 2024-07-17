import { createBrowserRouter, RouterProvider } from "react-router-dom";
import { Layout } from "../components/Layout";
import { Ws } from "../pages/Ws";
import { Http } from "../pages/Http";
import { Mqtt } from "../pages/Mqtt";

//@ts-ignore
const router = createBrowserRouter(
  [
    {
      path: "/",
      element: <Layout />,
      children: [
        // {
        //   path: "/",
        //   element: <Home />,
        // },
        {
          path: "/ws",
          element: <Ws />,
        },
        {
          path: "/http",
          index: true,
          element: <Http />,
        },
        {
          path: "/mqtt",
          element: <Mqtt />,
        },
      ],
    },
  ],
  {
    basename: "/rust-embedded-study",
  }
);

export const Routers = () => {
  return <RouterProvider router={router}></RouterProvider>;
};