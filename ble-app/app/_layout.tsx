import { GluestackUIProvider } from "@/components/ui/gluestack-ui-provider";
import { Stack } from "expo-router";
import { SafeAreaProvider } from "react-native-safe-area-context";
import { GestureHandlerRootView } from "react-native-gesture-handler";
import "../global.css";

export default function RootLayout() {
  return (
    <SafeAreaProvider>
      <GluestackUIProvider>
        <GestureHandlerRootView style={{ flex: 1 }}>
          <Stack
            screenOptions={{
              headerShown: false,
              animation: "slide_from_right",
              headerTitleAlign: "center",
            }}
          ></Stack>
        </GestureHandlerRootView>
      </GluestackUIProvider>
    </SafeAreaProvider>
  );
}
