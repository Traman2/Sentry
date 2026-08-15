import { MessageSquare, Plus } from "lucide-react";
import { useEffect, useMemo } from "react";
import { PanelEmpty, PanelListItem, PanelSection } from "@/components/PanelList";
import { Button } from "@/components/ui/button";
import { groupByRecency } from "@/lib/relativeTime";
import { useChatStore, type ChatSpace } from "@/store/chat";
import { useTabStore } from "@/store/tabs";

function Chatbot() {
  const chatSpaces = useChatStore((state) => state.chatSpaces);
  const loaded = useChatStore((state) => state.loaded);
  const refresh = useChatStore((state) => state.refresh);
  const createChatSpace = useChatStore((state) => state.createChatSpace);
  const deleteChatSpace = useChatStore((state) => state.deleteChatSpace);
  const openTab = useTabStore((state) => state.openTab);
  const discardTab = useTabStore((state) => state.discardTab);

  useEffect(() => {
    if (!loaded) refresh();
  }, [loaded, refresh]);

  // The store keeps spaces most-recently-active first, which is exactly the
  // order `groupByRecency` needs to emit each date heading once.
  const groups = useMemo(
    () => groupByRecency(chatSpaces, (space) => space.updated_at_ms),
    [chatSpaces],
  );

  const openChatSpace = (space: ChatSpace) => {
    openTab({ id: String(space.id), type: "chat-space", title: space.title });
  };

  const handleNewChat = async () => {
    openChatSpace(await createChatSpace());
  };

  const handleDelete = async (space: ChatSpace) => {
    await deleteChatSpace(space.id);
    discardTab(String(space.id));
  };

  return (
    <div className="flex flex-col gap-3">
      <Button size="sm" className="w-full py-4" onClick={handleNewChat}>
        <Plus />
        New chat
      </Button>

      {chatSpaces.length === 0 ? (
        <PanelEmpty
          icon={<MessageSquare />}
          title="No chats yet"
          description="Start one above, or ask about a process from the Resource Monitor."
        />
      ) : (
        groups.map((group) => (
          <PanelSection key={group.label} label={group.label}>
            {group.items.map((space) => (
              <PanelListItem
                key={space.id}
                title={space.title}
                onOpen={() => openChatSpace(space)}
                onDelete={() => handleDelete(space)}
                deleteLabel={`Delete chat "${space.title}"`}
              />
            ))}
          </PanelSection>
        ))
      )}
    </div>
  );
}

export default Chatbot;
