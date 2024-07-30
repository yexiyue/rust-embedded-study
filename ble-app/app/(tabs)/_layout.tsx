import { Tabs } from "expo-router";
import { ServerIcon, LightbulbIcon } from "lucide-react-native";
export default function TabLayout() {
  return (
    <Tabs
      screenOptions={{
        tabBarActiveTintColor: "green",
        headerTitleAlign: "center",
      }}
    >
      <Tabs.Screen
        name="index"
        options={{
          title: "灯光设置",
          tabBarIcon: (props) => <LightbulbIcon {...props} />,
        }}
      ></Tabs.Screen>
      <Tabs.Screen
        name="services"
        options={{
          title: "蓝牙服务",
          tabBarIcon: (props) => <ServerIcon {...props} />,
        }}
      ></Tabs.Screen>
    </Tabs>
  );
}
