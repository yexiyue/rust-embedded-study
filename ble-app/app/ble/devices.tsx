import {
  AlertDialog,
  AlertDialogBackdrop,
  AlertDialogBody,
  AlertDialogCloseButton,
  AlertDialogContent,
  AlertDialogFooter,
  AlertDialogHeader,
} from "@/components/ui/alert-dialog";
import { Box } from "@/components/ui/box";
import { Button, ButtonText } from "@/components/ui/button";
import { Heading } from "@/components/ui/heading";
import { HStack } from "@/components/ui/hstack";
import { Icon } from "@/components/ui/icon";
import { Text } from "@/components/ui/text";
import { VStack } from "@/components/ui/vstack";
import { manager, useBle } from "@/hooks/useBle";
import { useBleStore } from "@/hooks/useBleStore";
import { FlashList } from "@shopify/flash-list";
import { useAsyncEffect } from "ahooks";
import { ActivityAction, startActivityAsync } from "expo-intent-launcher";
import { Link, Stack } from "expo-router";
import { Bluetooth, X } from "lucide-react-native";
import React, { useEffect, useState } from "react";
import { State } from "react-native-ble-plx";
export default function BleDevices() {
  const { devices, requestPermissions, stopScanDevice, startScanDevices } =
    useBle();
  const [clearDevice] = useBleStore((state) => [state.clearDevice]);
  const [showAlertDialog, setShowAlertDialog] = useState(false);
  const [bleState, setBleState] = useState<State>();
  useAsyncEffect(async () => {
    const state = await manager.state();
    setBleState(state);
  }, []);
  useEffect(() => {
    if (bleState === State.PoweredOn) {
      startScanDevices();
    }
    return () => {
      stopScanDevice();
    };
  }, [bleState]);

  return (
    <>
      <Stack.Screen
        options={{
          title: "Devices",
          headerShown: true,
        }}
      />
      <AlertDialog
        isOpen={showAlertDialog}
        onClose={() => {
          setShowAlertDialog(false);
        }}
        size={"sm"}
      >
        <AlertDialogBackdrop />
        <AlertDialogContent>
          <AlertDialogHeader>
            <Heading>提示</Heading>
            <AlertDialogCloseButton>
              <Icon as={X} size="lg" />
            </AlertDialogCloseButton>
          </AlertDialogHeader>
          <AlertDialogBody>
            <Text>当前蓝牙未开启，是否开启蓝牙？</Text>
          </AlertDialogBody>
          <AlertDialogFooter className="gap-3">
            <Button
              variant="outline"
              action="secondary"
              onPress={() => {
                setShowAlertDialog(false);
              }}
            >
              <ButtonText>取消</ButtonText>
            </Button>
            <Button
              action="primary"
              onPress={async () => {
                await startActivityAsync(ActivityAction.BLUETOOTH_SETTINGS);
                setBleState(await manager.state());
                setShowAlertDialog(false);
              }}
            >
              <ButtonText>开启</ButtonText>
            </Button>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      <Box className="flex w-full h-full bg-white">
        <FlashList
          contentContainerStyle={{
            padding: 16,
          }}
          data={bleState === State.PoweredOn ? devices : []}
          renderItem={({ item }) => {
            return (
              <HStack
                key={item.id}
                className="border border-b-0 border-gray-300 last:border-b-[1px] mb-2 rounded-lg py-2 px-4 justify-between items-center relative"
              >
                <Bluetooth
                  size={24}
                  color="green"
                  style={{
                    position: "absolute",
                    left: 8,
                  }}
                />
                <Box className="ml-8">
                  <Text bold className="text-xl">
                    {item.name || "Unknown"}
                  </Text>
                  <Text>MAC: {item.id}</Text>
                </Box>
                {item.isConnectable && (
                  <Link
                    href={`/ble/connect/${item.id}`}
                    asChild
                    onPress={stopScanDevice}
                  >
                    <Button
                      action={"positive"}
                      variant={"solid"}
                      isDisabled={false}
                    >
                      <ButtonText>连接</ButtonText>
                    </Button>
                  </Link>
                )}
              </HStack>
            );
          }}
          refreshing={false}
          onRefresh={async () => {
            let res = await requestPermissions();
            let state = await manager.state();
            setBleState(state);
            if (res && state == State.PoweredOn) {
              startScanDevices();
            } else if (res && state === State.PoweredOff) {
              setShowAlertDialog(true);
              stopScanDevice();
            } else {
              stopScanDevice();
            }
            clearDevice();
          }}
          ListEmptyComponent={() => {
            return (
              <VStack className="w-full h-[80vh] justify-center items-center">
                {getBleText(bleState)}
                <Text>请下拉刷新</Text>
              </VStack>
            );
          }}
          estimatedItemSize={100}
        />
      </Box>
    </>
  );
}

function getBleText(bleState?: State) {
  if (bleState === State.PoweredOn) {
    return <Text>暂无设备</Text>;
  } else if (bleState === State.Unauthorized) {
    return <Text>未授权</Text>;
  } else if (bleState === State.Unsupported) {
    return <Text>不支持蓝牙</Text>;
  } else if (bleState === State.PoweredOff) {
    return <Text>蓝牙未开启</Text>;
  } else if (bleState === State.Unknown) {
    return <Text>未知错误</Text>;
  } else if (bleState === State.Resetting) {
    return <Text>蓝牙正在重置</Text>;
  } else {
    return <Text>蓝牙状态未知</Text>;
  }
}
