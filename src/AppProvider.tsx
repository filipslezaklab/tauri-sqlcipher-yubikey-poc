import { MantineProvider } from "@mantine/core";
import { ModalsProvider } from "@mantine/modals";
import { Notifications } from "@mantine/notifications";
import { ReactNode } from "@tanstack/react-router";
import { appTheme } from "./theme";

type Props = {
  children: ReactNode;
};

export const AppProvider = ({ children }: Props) => {
  return (
    <MantineProvider theme={appTheme} defaultColorScheme="light">
      <ModalsProvider>{children}</ModalsProvider>
      <Notifications />
    </MantineProvider>
  );
};
