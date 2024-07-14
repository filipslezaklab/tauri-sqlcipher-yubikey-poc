import { createRoute } from "@tanstack/react-router";
import { rootRoute } from "../../RouterTree";
import { SelectYubikeyModal } from "./components/SelectYubikey/SelectYubikey";
import "./style.scss";

const SplashPage = () => {
  return (
    <div id="splash-page">
      <h1>SQLCipher POC</h1>
      {/* <LoadingOverlay visible={true} overlayProps={{ radius: undefined, blur: 4 }} /> */}
      <SelectYubikeyModal />
    </div>
  );
};

export const splashRoute = createRoute({
  path: "/",
  getParentRoute: () => rootRoute,
  component: SplashPage,
});
