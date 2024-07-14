import { Button, Modal } from "@mantine/core";
import { useDisclosure } from "@mantine/hooks";
import { notifications } from "@mantine/notifications";
import { useNavigate } from "@tanstack/react-router";
import { clsx } from "clsx";
import { useEffect, useState } from "react";
import { initDatabase, listYk, selectYubikey } from "../../../../bindings";

export const SelectYubikeyModal = () => {
  const [opened, { open, close }] = useDisclosure(false);

  useEffect(() => {
    open();
    /* eslint-disable react-hooks/exhaustive-deps */
  }, []);

  return (
    <Modal opened={opened} onClose={close} title="Select Yubikey">
      <ModalContent />
    </Modal>
  );
};

const ModalContent = () => {
  const navigate = useNavigate();
  const [selected, setSelected] = useState<string | undefined>();
  const [pin, setPin] = useState<string>("");
  const [keys, setKeys] = useState<string[]>([]);
  const [loading, setLoading] = useState(false);

  const handleSubmit = () => {
    if (selected && pin) {
      setLoading(true);
      selectYubikey({
        pin: pin,
        serial: selected,
      })
        .then(() => {
          notifications.show({
            message: "Yubikey selected.",
          });
          initDatabase()
            .then(() => {
              notifications.show({
                message: "Database connection established.",
              });
              navigate({ to: "/db" });
            })
            .catch((e: string) => {
              setLoading(false);
              notifications.show({
                message: String(e),
              });
              console.error(e);
            });
        })
        .catch((e: string) => {
          setLoading(false);
          notifications.show({
            color: "red",
            message: String(e),
          });
        });
    }
  };

  useEffect(() => {
    setLoading(true);
    listYk()
      .then((res) => {
        setKeys(res);
      })
      .finally(() => setLoading(false));
  }, []);

  return (
    <>
      <div className="top">
        <label htmlFor="pin-input">KeyPin: </label>
        <input
          type="text"
          value={pin}
          onChange={(e) => {
            const parsed = parseInt(e.target.value);
            if (typeof parsed === "number" || e.target.value === "") {
              setPin(e.target.value.trim());
            }
          }}
          id="pin-input"
          disabled={loading}
        />
      </div>
      <ul>
        {keys.map((k) => (
          <li
            className={clsx({
              active: selected === k,
            })}
            key={k}
            onClick={() => setSelected(k)}
          >
            {k}
          </li>
        ))}
      </ul>
      <div className="controls">
        <Button
          variant="filled"
          fullWidth
          onClick={() => {
            if (pin && selected) {
              handleSubmit();
            }
          }}
          disabled={selected === undefined || pin === ""}
          loading={loading}
        >
          Submit
        </Button>
      </div>
    </>
  );
};
