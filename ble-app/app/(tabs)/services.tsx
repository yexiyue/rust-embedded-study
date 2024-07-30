import {
  Accordion,
  AccordionContent,
  AccordionHeader,
  AccordionIcon,
  AccordionItem,
  AccordionTitleText,
  AccordionTrigger,
} from "@/components/ui/accordion";
import { Alert, AlertIcon, AlertText } from "@/components/ui/alert";
import { Badge, BadgeIcon, BadgeText } from "@/components/ui/badge";
import { Box } from "@/components/ui/box";
import { Button, ButtonIcon, ButtonText } from "@/components/ui/button";
import { Center } from "@/components/ui/center";
import { Heading } from "@/components/ui/heading";
import { HStack } from "@/components/ui/hstack";
import { CloseIcon, Icon } from "@/components/ui/icon";
import {
  Modal,
  ModalBackdrop,
  ModalBody,
  ModalCloseButton,
  ModalContent,
  ModalFooter,
  ModalHeader,
} from "@/components/ui/modal";
import {
  Radio,
  RadioGroup,
  RadioIcon,
  RadioIndicator,
  RadioLabel,
} from "@/components/ui/radio";
import { Text } from "@/components/ui/text";
import { VStack } from "@/components/ui/vstack";
import { useStore } from "@/hooks/useStore";
import { FlashList } from "@shopify/flash-list";
import { useRouter } from "expo-router";
import {
  ChevronDownIcon,
  ChevronUpIcon,
  CircleIcon,
  InfoIcon,
  LightbulbOff,
  Palette,
  Plus,
} from "lucide-react-native";
import React, { useMemo, useState } from "react";
export default function Settings() {
  const [showModal, setShowModal] = useState(false);
  const router = useRouter();
  const [current, services, removeService, setCurrent] = useStore((store) => [
    store.current,
    store.services,
    store.removeService,
    store.setCurrent,
  ]);

  const servicesArray = useMemo(() => {
    return Object.entries(services).map(([key, value]) => ({
      deviceId: key,
      ...value,
    }));
  }, [services]);

  const [deviceInfo, setDeviceInfo] = useState<{
    name?: string | null;
    deviceId?: string;
  }>({});

  return (
    <>
      <VStack className=" bg-white w-full h-full relative">
        <RadioGroup
          value={current}
          onChange={(value) => {
            setCurrent(value);
          }}
        >
          <Accordion
            className="w-full h-full"
            variant={"filled"}
            type={"single"}
            isDisabled={false}
          >
            <FlashList
              data={servicesArray}
              renderItem={({ item }) => {
                return (
                  <AccordionItem key={item.deviceId} value={item.deviceId}>
                    <AccordionHeader className="border-b border-gray-300">
                      <AccordionTrigger>
                        {({ isExpanded }: { isExpanded: boolean }) => {
                          return (
                            <>
                              <Radio value={item.deviceId} size="md">
                                <RadioIndicator>
                                  <RadioIcon as={CircleIcon} />
                                </RadioIndicator>
                                <RadioLabel>
                                  <AccordionTitleText>
                                    {item.name || "Unknown"}
                                  </AccordionTitleText>
                                </RadioLabel>
                              </Radio>
                              {isExpanded ? (
                                <AccordionIcon as={ChevronUpIcon} />
                              ) : (
                                <AccordionIcon as={ChevronDownIcon} />
                              )}
                            </>
                          );
                        }}
                      </AccordionTrigger>
                    </AccordionHeader>
                    <AccordionContent>
                      <VStack className="w-full gap-4">
                        <Alert action="info" variant="solid">
                          <AlertIcon as={InfoIcon} />
                          <AlertText>Uuid: {item.deviceId}</AlertText>
                        </Alert>
                        <Box>
                          <HStack className="gap-2">
                            <Badge
                              variant={"solid"}
                              action={"warning"}
                              size={"md"}
                            >
                              <BadgeIcon as={LightbulbOff} />
                              <BadgeText className="ml-2">关灯服务</BadgeText>
                            </Badge>
                          </HStack>

                          <VStack>
                            <Text>ServiceUuid:</Text>
                            <Text className="text-sm">
                              {item.closeLightService?.serviceUuid}
                            </Text>
                          </VStack>
                          <VStack>
                            <Text>CharacteristicUuid:</Text>
                            <Text className="text-sm">
                              {item.closeLightService?.characteristicUuid}
                            </Text>
                          </VStack>
                        </Box>
                        <Box>
                          <HStack className="gap-2">
                            <Badge
                              variant={"solid"}
                              action={"info"}
                              size={"md"}
                            >
                              <BadgeIcon as={Palette} />
                              <BadgeText className="ml-2">
                                灯光颜色服务
                              </BadgeText>
                            </Badge>
                          </HStack>

                          <VStack>
                            <Text>ServiceUuid:</Text>
                            <Text className="text-sm">
                              {item.setLightService?.serviceUuid}
                            </Text>
                          </VStack>
                          <VStack>
                            <Text>CharacteristicUuid:</Text>
                            <Text className="text-sm">
                              {item.setLightService?.characteristicUuid}
                            </Text>
                          </VStack>
                        </Box>
                        <Button
                          action={"negative"}
                          variant={"solid"}
                          onPress={() => {
                            setDeviceInfo({
                              name: item.name,
                              deviceId: item.deviceId,
                            });
                            setShowModal(true);
                          }}
                        >
                          <ButtonText>删除服务</ButtonText>
                        </Button>
                      </VStack>
                    </AccordionContent>
                  </AccordionItem>
                );
              }}
              estimatedItemSize={200}
              ListEmptyComponent={() => {
                return (
                  <VStack className="w-full h-[80vh] justify-center items-center">
                    <Text className="mb-2">暂无服务</Text>
                    <Button
                      action={"positive"}
                      variant={"solid"}
                      onPress={() => {
                        router.push("/ble/devices");
                      }}
                    >
                      <ButtonIcon as={Plus} />
                      <ButtonText>添加服务</ButtonText>
                    </Button>
                  </VStack>
                );
              }}
            />
          </Accordion>
        </RadioGroup>
        <Button
          action={"positive"}
          variant={"solid"}
          className="absolute rounded-full w-[50px] h-[50px] bottom-6 right-6"
          onPress={() => {
            router.push("/ble/devices");
          }}
        >
          <ButtonIcon as={Plus} />
        </Button>
      </VStack>
      <Center>
        <Modal
          isOpen={showModal}
          onClose={() => {
            setShowModal(false);
          }}
        >
          <ModalBackdrop />
          <ModalContent>
            <ModalHeader>
              <Heading size="lg">警告</Heading>
              <ModalCloseButton>
                <Icon as={CloseIcon} />
              </ModalCloseButton>
            </ModalHeader>
            <ModalBody>
              <Text>
                确定要删除设备{deviceInfo.name || "Unknown"} (
                {deviceInfo.deviceId})的蓝牙服务吗？
                删除后将无法通过蓝牙控制该设备上的LED灯。
              </Text>
            </ModalBody>
            <ModalFooter>
              <Button
                variant="outline"
                size="sm"
                action="secondary"
                className="mr-3"
                onPress={() => {
                  setShowModal(false);
                }}
              >
                <ButtonText>取消</ButtonText>
              </Button>
              <Button
                size="sm"
                action="negative"
                className="border-0"
                onPress={() => {
                  setShowModal(false);
                  if (deviceInfo.deviceId) {
                    removeService(deviceInfo.deviceId);
                  }
                }}
              >
                <ButtonText>确定</ButtonText>
              </Button>
            </ModalFooter>
          </ModalContent>
        </Modal>
      </Center>
    </>
  );
}
