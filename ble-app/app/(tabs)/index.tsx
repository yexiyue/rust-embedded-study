import {
  Actionsheet,
  ActionsheetBackdrop,
  ActionsheetContent,
  ActionsheetDragIndicator,
  ActionsheetDragIndicatorWrapper,
} from "@/components/ui/actionsheet";
import { Alert, AlertIcon, AlertText } from "@/components/ui/alert";
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
import { Center } from "@/components/ui/center";
import { Heading } from "@/components/ui/heading";
import { Icon } from "@/components/ui/icon";
import { Text } from "@/components/ui/text";
import {
  Toast,
  ToastDescription,
  ToastTitle,
  useToast,
} from "@/components/ui/toast";
import { VStack } from "@/components/ui/vstack";
import { manager, uint8ArrayToBase64, useBle } from "@/hooks/useBle";
import { useStore } from "@/hooks/useStore";
import { useAsyncEffect, useMemoizedFn } from "ahooks";
import { ActivityAction, startActivityAsync } from "expo-intent-launcher";
import { useRouter } from "expo-router";
import { InfoIcon, Lightbulb, LightbulbOff, X } from "lucide-react-native";
import React, { useMemo, useRef, useState } from "react";
import { Pressable } from "react-native";
import { Device, State } from "react-native-ble-plx";
import { RefreshControl, ScrollView } from "react-native-gesture-handler";
import ColorPicker, {
  BrightnessSlider,
  colorKit,
  HueSlider,
  Panel1,
  PreviewText,
  Swatches,
} from "reanimated-color-picker";

export default function Home() {
  const router = useRouter();
  const toast = useToast();
  const { requestPermissions, connectDevice } = useBle();
  const [showAlertDialog, setShowAlertDialog] = useState(false);
  const [refreshing, setRefreshing] = useState(false);
  const deviceRef = useRef<Device>();
  const [isClose, setIsClose] = useState(false);
  const [showActionsheet, setShowActionsheet] = React.useState(false);
  const [services, current, setDeviceColor] = useStore((store) => [
    store.services,
    store.current,
    store.setColor,
  ]);

  const [color, setColor] = useState<string>(
    services[current]?.color ?? "#ff2442"
  );

  const getStateAndConnect = useMemoizedFn(async () => {
    setRefreshing(true);
    if (deviceRef.current?.id !== current) {
      await deviceRef.current?.cancelConnection();
    }
    const state = await manager.state();
    const allow = await requestPermissions();
    if (!allow) {
      toast.show({
        duration: 2000,
        placement: "top",
        render: (props) => {
          return (
            <Toast
              nativeID="permission-denied"
              action="error"
              variant="solid"
              {...props}
            >
              <ToastTitle>没有蓝牙权限</ToastTitle>
              <ToastDescription>请到设置中启用蓝牙权限</ToastDescription>
            </Toast>
          );
        },
      });
      setRefreshing(false);
      return;
    }
    if (current && state === State.PoweredOn) {
      try {
        if (!(await deviceRef.current?.isConnected())) {
          const device = await connectDevice(current);
          deviceRef.current = device;
        }
      } catch (error) {
        toast.show({
          duration: 2000,
          placement: "top",
          render: (props) => {
            return (
              <Toast
                nativeID="connect-error"
                action="error"
                variant="solid"
                {...props}
              >
                <ToastTitle>连接失败</ToastTitle>
                <ToastDescription>{`${error}`}</ToastDescription>
              </Toast>
            );
          },
        });
      }
    }
    if (state === State.PoweredOff) {
      setShowAlertDialog(true);
    }
    setRefreshing(false);
  });

  const currentService = useMemo(() => {
    return services[current];
  }, [current]);

  useAsyncEffect(async () => {
    getStateAndConnect();
  }, [current]);

  return (
    <>
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
                setShowAlertDialog(false);
              }}
            >
              <ButtonText>开启</ButtonText>
            </Button>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
      <ScrollView
        style={{ flex: 1, height: "100%", backgroundColor: "white" }}
        refreshControl={
          <RefreshControl
            refreshing={refreshing}
            onRefresh={() => {
              getStateAndConnect();
            }}
          />
        }
      >
        {currentService ? (
          <Box className="w-full h-full gap-4 bg-white items-center">
            <Box className="w-[80%] mt-20">
              <Alert
                className="h-[80px] flex flex-col"
                action="info"
                variant="solid"
              >
                <AlertText>
                  设备名: {currentService.name || "Unknown"}{" "}
                </AlertText>
                <AlertText>Uuid: {current}</AlertText>
              </Alert>
            </Box>
            <Pressable
              onPress={async () => {
                const { characteristicUuid, serviceUuid } =
                  currentService.closeLightService;
                if (await deviceRef.current?.isConnected()) {
                  try {
                    await deviceRef.current?.writeCharacteristicWithoutResponseForService(
                      serviceUuid,
                      characteristicUuid,
                      uint8ArrayToBase64([1])
                    );
                    setIsClose(true);
                  } catch (error) {
                    toast.show({
                      duration: 2000,
                      placement: "top",
                      render: (props) => {
                        return (
                          <Toast
                            nativeID="connect-error"
                            action="warning"
                            variant="solid"
                            {...props}
                          >
                            <ToastTitle>未知错误</ToastTitle>
                            <ToastDescription>{`${error}`}</ToastDescription>
                          </Toast>
                        );
                      },
                    });
                  }
                } else {
                  toast.show({
                    duration: 2000,
                    placement: "top",
                    render: (props) => {
                      return (
                        <Toast
                          nativeID="connect-error"
                          action="warning"
                          variant="solid"
                          {...props}
                        >
                          <ToastTitle>提示</ToastTitle>
                          <ToastDescription>请先连接设备</ToastDescription>
                        </Toast>
                      );
                    },
                  });
                }
              }}
            >
              <Center className="w-[150px] h-[150px]">
                {isClose ? (
                  <LightbulbOff
                    size={100}
                    color={services[current]?.color ?? "#ff2442"}
                  />
                ) : (
                  <Lightbulb
                    size={100}
                    color={services[current]?.color ?? "#ff2442"}
                  />
                )}
              </Center>
            </Pressable>
            <Button
              action={"primary"}
              variant={"solid"}
              size={"lg"}
              isDisabled={false}
              onPress={() => {
                setShowActionsheet(true);
              }}
            >
              <ButtonText>设置灯光颜色</ButtonText>
            </Button>
          </Box>
        ) : (
          <Box className="w-full h-full gap-4 bg-white items-center">
            <Box className="w-[80%] mt-4">
              <Alert action="warning" variant="solid">
                <AlertIcon as={InfoIcon} />
                <AlertText>暂无可用服务</AlertText>
              </Alert>
            </Box>
            <Button
              action={"positive"}
              variant={"solid"}
              isDisabled={false}
              onPress={() => {
                router.push("ble/devices");
              }}
            >
              <ButtonText>添加服务</ButtonText>
            </Button>
          </Box>
        )}
        <Actionsheet
          isOpen={showActionsheet}
          onClose={() => {
            setShowActionsheet(false);
          }}
        >
          <ActionsheetBackdrop />
          <ActionsheetContent>
            <ActionsheetDragIndicatorWrapper>
              <ActionsheetDragIndicator />
            </ActionsheetDragIndicatorWrapper>
            <VStack className={`w-full p-4 gap-4`}>
              <ColorPicker
                value={color}
                onComplete={(color) => {
                  setColor(color.rgb);
                }}
              >
                <VStack className="gap-4">
                  <PreviewText colorFormat="rgb" />
                  <Panel1 />
                  <HueSlider />
                  <BrightnessSlider />
                  <Swatches />
                </VStack>
              </ColorPicker>
              <Box>
                <Button
                  action={"primary"}
                  className="w-full"
                  variant={"solid"}
                  onPress={async () => {
                    const { characteristicUuid, serviceUuid } =
                      currentService.setLightService;
                    const rgb = colorKit.RGB(color).object();

                    if (await deviceRef.current?.isConnected()) {
                      try {
                        await deviceRef.current?.writeCharacteristicWithoutResponseForService(
                          serviceUuid,
                          characteristicUuid,
                          uint8ArrayToBase64([rgb.r, rgb.g, rgb.b])
                        );
                        setIsClose(false);
                        setDeviceColor(color);
                      } catch (error) {
                        toast.show({
                          duration: 2000,
                          placement: "top",
                          render: (props) => {
                            return (
                              <Toast
                                nativeID="connect-error"
                                action="warning"
                                variant="solid"
                                {...props}
                              >
                                <ToastTitle>未知错误</ToastTitle>
                                <ToastDescription>
                                  {`${error}`}
                                </ToastDescription>
                              </Toast>
                            );
                          },
                        });
                      }
                    } else {
                      toast.show({
                        duration: 2000,
                        placement: "top",
                        render: (props) => {
                          return (
                            <Toast
                              nativeID="connect-error"
                              action="warning"
                              variant="solid"
                              {...props}
                            >
                              <ToastTitle>警告</ToastTitle>
                              <ToastDescription>请先连接设备</ToastDescription>
                            </Toast>
                          );
                        },
                      });
                    }
                    setShowActionsheet(false);
                  }}
                >
                  <ButtonText>设置颜色</ButtonText>
                </Button>
              </Box>
            </VStack>
          </ActionsheetContent>
        </Actionsheet>
      </ScrollView>
    </>
  );
}
