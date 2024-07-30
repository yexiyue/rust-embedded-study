import {
  Accordion,
  AccordionContent,
  AccordionHeader,
  AccordionIcon,
  AccordionItem,
  AccordionTitleText,
  AccordionTrigger,
} from "@/components/ui/accordion";
import {
  Actionsheet,
  ActionsheetBackdrop,
  ActionsheetContent,
  ActionsheetDragIndicator,
  ActionsheetDragIndicatorWrapper,
  ActionsheetItem,
  ActionsheetItemText,
} from "@/components/ui/actionsheet";
import { Badge, BadgeIcon, BadgeText } from "@/components/ui/badge";
import { Box } from "@/components/ui/box";
import { Button, ButtonText } from "@/components/ui/button";
import { Center } from "@/components/ui/center";
import { Divider } from "@/components/ui/divider";
import { HStack } from "@/components/ui/hstack";
import { Text } from "@/components/ui/text";
import { VStack } from "@/components/ui/vstack";
import { manager } from "@/hooks/useBle";
import { useBleStore } from "@/hooks/useBleStore";
import { ServiceConfig, useStore } from "@/hooks/useStore";
import { FlashList } from "@shopify/flash-list";
import { useMemoizedFn } from "ahooks";
import { Stack, useLocalSearchParams, useRouter } from "expo-router";
import {
  Bell,
  BookCheck,
  ChevronDownIcon,
  ChevronUpIcon,
  LightbulbOff,
  Palette,
  PencilLine,
  Server,
  ServerOff,
  Waypoints,
} from "lucide-react-native";
import React, { useEffect, useMemo, useState } from "react";
import { Characteristic, Device, Service } from "react-native-ble-plx";

export default function Connect() {
  const [showActionsheet, setShowActionsheet] = React.useState(false);
  const [clearDevice] = useBleStore((store) => [store.clearDevice]);
  // 保存配置到store
  const [setService] = useStore((store) => [store.setService]);
  const { id } = useLocalSearchParams();
  const router = useRouter();
  const [services, setServices] = useState<Service[]>([]);
  const [characteristics, setCharacteristics] = useState<{
    [key: Service["uuid"]]: Characteristic[];
  }>({});
  const [isLoading, setIsLoading] = useState(false);

  const [current, setCurrent] = useState<{
    service: Service;
    characteristic: Characteristic;
  }>();

  const [servicesConfig, setServicesConfig] = useState<Partial<ServiceConfig>>(
    {}
  );
  // 获取服务和特征信息
  const getData = useMemoizedFn(async () => {
    setIsLoading(true);
    const device = await manager.connectToDevice(id as string);
    await device.discoverAllServicesAndCharacteristics();

    const services = await device.services();
    let tasks = services.map((service) => {
      return service.characteristics().then((characteristics) => {
        return [service.uuid, characteristics];
      });
    });
    let res = (await Promise.allSettled(tasks)).map((item) =>
      item.status === "fulfilled" ? item.value : undefined
    );
    setServicesConfig({
      name: device.name,
    });
    setCharacteristics(Object.fromEntries(res as any));
    setServices(services);
    await device.cancelConnection();
    setIsLoading(false);
    
  });

  const isCompleted = useMemo(() => {
    return servicesConfig.setLightService && servicesConfig.closeLightService;
  }, [servicesConfig]);

  useEffect(() => {
    getData();
  }, [id]);

  return (
    <>
      <Stack.Screen
        options={{
          title: "Services",
          headerShown: true,
          headerRight: (props) => {
            return (
              <Button
                onPress={() => {
                  setShowActionsheet(true);
                }}
                action="positive"
                variant="link"
                disabled={isLoading || !isCompleted}
              >
                <ButtonText
                  className={`${!isCompleted ? "text-gray-400" : ""}`}
                >
                  完成
                </ButtonText>
              </Button>
            );
          },
        }}
      />
      <Box className="flex w-full h-full bg-white">
        <Accordion
          className="w-full h-full"
          variant={"filled"}
          type={"multiple"}
          isDisabled={false}
        >
          <FlashList
            data={services}
            renderItem={({ item }) => {
              return (
                <AccordionItem key={item.uuid} value={item.uuid}>
                  <AccordionHeader className="border-b border-gray-300">
                    <AccordionTrigger>
                      {(states: any) => (
                        <>
                          <Server size={18} color={"green"} />
                          <AccordionTitleText className="text-base ml-4">
                            {item.uuid}
                          </AccordionTitleText>
                          {states.isExpanded ? (
                            <AccordionIcon as={ChevronUpIcon} />
                          ) : (
                            <AccordionIcon as={ChevronDownIcon} />
                          )}
                        </>
                      )}
                    </AccordionTrigger>
                  </AccordionHeader>
                  <AccordionContent>
                    <Box className="ml-4">
                      {characteristics[item.uuid]?.map((characteristic) => {
                        const isDisabled =
                          !characteristic.isWritableWithoutResponse &&
                          !characteristic.isWritableWithResponse;
                        return (
                          <Box key={characteristic.id}>
                            <Box>
                              <VStack>
                                <Text bold className="text-base">
                                  Characteristic
                                </Text>
                                <Text>{characteristic.uuid}</Text>
                              </VStack>
                              <HStack className=" gap-2">
                                {characteristic.isReadable && (
                                  <Badge
                                    variant={"solid"}
                                    action={"info"}
                                    size={"md"}
                                  >
                                    <BadgeText>Read</BadgeText>
                                    <BadgeIcon
                                      as={BookCheck}
                                      className="ml-2"
                                    />
                                  </Badge>
                                )}
                                {(characteristic.isWritableWithResponse ||
                                  characteristic.isWritableWithoutResponse) && (
                                  <Badge
                                    variant={"solid"}
                                    action={"success"}
                                    size={"md"}
                                  >
                                    <BadgeText>Write</BadgeText>
                                    <BadgeIcon
                                      as={PencilLine}
                                      className="ml-2"
                                    />
                                  </Badge>
                                )}
                                {characteristic.isNotifying && (
                                  <Badge
                                    variant={"solid"}
                                    action={"warning"}
                                    size={"md"}
                                  >
                                    <BadgeText>Notify</BadgeText>
                                    <BadgeIcon as={Bell} className="ml-2" />
                                  </Badge>
                                )}
                                {characteristic.isIndicatable && (
                                  <Badge
                                    variant={"solid"}
                                    action={"muted"}
                                    size={"md"}
                                  >
                                    <BadgeText>Indicate</BadgeText>
                                    <BadgeIcon
                                      as={Waypoints}
                                      className="ml-2"
                                    />
                                  </Badge>
                                )}
                              </HStack>
                            </Box>
                            <Button
                              action={"positive"}
                              variant={"solid"}
                              isDisabled={false}
                              className={`mt-4 ${isDisabled ? "opacity-50" : ""}`}
                              disabled={isDisabled}
                              onPress={() => {
                                setCurrent({
                                  service: item,
                                  characteristic: characteristic,
                                });
                                setShowActionsheet(true);
                              }}
                            >
                              <ButtonText>添加设置</ButtonText>
                            </Button>
                            <Divider className="mt-2" />
                          </Box>
                        );
                      })}
                    </Box>
                  </AccordionContent>
                </AccordionItem>
              );
            }}
            refreshing={isLoading}
            onRefresh={async () => {
              setServices([]);
              await getData();
            }}
            ListEmptyComponent={() => {
              return (
                <Center className="w-full h-[80vh]">
                  <ServerOff size={20} color={"gray"} />
                  <Text>暂无服务</Text>
                  <Text>请下拉刷新</Text>
                </Center>
              );
            }}
            estimatedItemSize={200}
          />
        </Accordion>
      </Box>
      {/* 配置服务 */}
      <Actionsheet
        isOpen={showActionsheet}
        onClose={() => setShowActionsheet(false)}
      >
        <ActionsheetBackdrop />
        <ActionsheetContent>
          <ActionsheetDragIndicatorWrapper>
            <ActionsheetDragIndicator />
          </ActionsheetDragIndicatorWrapper>
          <Box className="h-[70%] w-full">
            <VStack>
              <ActionsheetItem
                onPress={() => {
                  if (current) {
                    setServicesConfig({
                      ...servicesConfig,
                      closeLightService: {
                        serviceUuid: current?.service.uuid,
                        characteristicUuid: current?.characteristic.uuid,
                      },
                    });
                  }
                  setShowActionsheet(false);
                }}
                isDisabled={
                  !current?.characteristic.isWritableWithResponse &&
                  !current?.characteristic.isWritableWithoutResponse
                }
              >
                <Center className="w-full flex-row gap-2">
                  <LightbulbOff size={14} color="gray" />
                  <ActionsheetItemText>设置为关灯服务</ActionsheetItemText>
                </Center>
              </ActionsheetItem>
              <ActionsheetItem
                onPress={() => {
                  if (current) {
                    setServicesConfig({
                      ...servicesConfig,
                      setLightService: {
                        serviceUuid: current?.service.uuid,
                        characteristicUuid: current?.characteristic.uuid,
                      },
                    });
                  }
                  setShowActionsheet(false);
                }}
                isDisabled={
                  !current?.characteristic.isWritableWithResponse &&
                  !current?.characteristic.isWritableWithoutResponse
                }
              >
                <Center className="w-full flex-row gap-2">
                  <Palette size={14} color="gray" />
                  <ActionsheetItemText>设置为灯光颜色服务</ActionsheetItemText>
                </Center>
              </ActionsheetItem>
            </VStack>
            <Divider />
            <VStack className="w-full gap-4 my-4">
              <Box>
                <HStack className="gap-2">
                  <Text bold>关灯服务</Text>
                  {servicesConfig?.closeLightService ? (
                    <Badge variant={"solid"} action={"info"} size={"md"}>
                      <BadgeText>已设置</BadgeText>
                    </Badge>
                  ) : (
                    <Badge variant={"solid"} action={"muted"} size={"md"}>
                      <BadgeText>未设置</BadgeText>
                    </Badge>
                  )}
                </HStack>
                {servicesConfig?.closeLightService && (
                  <>
                    <VStack>
                      <Text>ServiceUuid:</Text>
                      <Text className="text-sm">
                        {servicesConfig.closeLightService?.serviceUuid}
                      </Text>
                    </VStack>
                    <VStack>
                      <Text>CharacteristicUuid:</Text>
                      <Text className="text-sm">
                        {servicesConfig.closeLightService?.characteristicUuid}
                      </Text>
                    </VStack>
                  </>
                )}
              </Box>
              <Box>
                <HStack className="gap-2">
                  <Text bold>灯光颜色服务</Text>
                  {servicesConfig?.closeLightService ? (
                    <Badge variant={"solid"} action={"info"} size={"md"}>
                      <BadgeText>已设置</BadgeText>
                    </Badge>
                  ) : (
                    <Badge variant={"solid"} action={"muted"} size={"md"}>
                      <BadgeText>未设置</BadgeText>
                    </Badge>
                  )}
                </HStack>
                {servicesConfig?.setLightService && (
                  <>
                    <VStack>
                      <Text>ServiceUuid:</Text>
                      <Text className="text-sm">
                        {servicesConfig.setLightService?.serviceUuid}
                      </Text>
                    </VStack>
                    <VStack>
                      <Text>CharacteristicUuid:</Text>
                      <Text className="text-sm">
                        {servicesConfig.setLightService?.characteristicUuid}
                      </Text>
                    </VStack>
                  </>
                )}
              </Box>
            </VStack>
            <Button
              action={"positive"}
              variant={"solid"}
              disabled={!isCompleted}
              className={`${!isCompleted ? "opacity-50" : ""}`}
              onPress={() => {
                if (id) {
                  setService(id as string, servicesConfig as any);
                }
                clearDevice();
                router.dismissAll();
              }}
            >
              <ButtonText>完成设置</ButtonText>
            </Button>
          </Box>
        </ActionsheetContent>
      </Actionsheet>
    </>
  );
}
