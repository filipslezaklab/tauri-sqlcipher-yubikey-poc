import { Button } from "@mantine/core";
import { createRoute } from "@tanstack/react-router";
import { useCallback, useEffect, useState } from "react";
import { addDbItem, listDb, MyFieldTs } from "../../bindings";
import { rootRoute } from "../../RouterTree";
import "./style.scss";

const DbPage = () => {
  const [items, setItems] = useState<MyFieldTs[]>([]);
  const [input, setInput] = useState("");

  const updateList = useCallback(() => {
    listDb().then((res) => setItems(res));
  }, [setItems]);

  const addItem = useCallback(
    (val: string) => {
      addDbItem({
        id: null,
        test_field: val,
      }).then(() => {
        updateList();
        setInput("");
      });
    },
    [setInput, updateList],
  );

  useEffect(() => {
    updateList();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return (
    <div id="db-page">
      <div className="controls">
        <label htmlFor="data-input"></label>
        <input id="data-input" value={input} onChange={(e) => setInput(e.target.value)} />
        <Button onClick={() => addItem(input)} disabled={input.length < 3}>
          Add item
        </Button>
      </div>
      <div className="content">
        <ul>
          {items.map((item) => (
            <li key={item.id}>
              <span>{item.test_field}</span>
            </li>
          ))}
        </ul>
      </div>
    </div>
  );
};

export const dbRoute = createRoute({
  path: "/db",
  component: DbPage,
  getParentRoute: () => rootRoute,
});
