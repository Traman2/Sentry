import { MessageSquare, Plus, Trash2 } from "lucide-react";
import { useEffect, type MouseEvent } from "react";
import { Button } from "@/components/ui/button";
import { useChatStore, type ChatSpace } from "@/store/chat";
import { useTabStore } from "@/store/tabs";

function openChatSpaceTab(
  openTab: (tab: { id: string; type: "chat-space"; title: string }) => void,
  space: ChatSpace,
) {
  openTab({ id: String(space.id), type: "chat-space", title: space.title });
}

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

  const handleNewChat = async () => {
    const space = await createChatSpace();
    openChatSpaceTab(openTab, space);
  };

  const handleDeleteChat = async (e: MouseEvent, space: ChatSpace) => {
    e.stopPropagation();
    await deleteChatSpace(space.id);
    discardTab(String(space.id));
  };

  return (
    <div className="flex h-full flex-col gap-1">
      <Button className="w-full rounded-lg" onClick={handleNewChat}>
        <Plus className="h-3.5 w-3.5" />
        New chat
      </Button>

      <p className="mt-2 text-[10px] font-semibold tracking-wide text-navy/60 uppercase">
        Chats
      </p>
      <div className="flex flex-col gap-0.5 overflow-y-auto">
        {chatSpaces.length === 0 && (
          <p className="text-xs text-muted-foreground">No chats yet.</p>
        )}
        {chatSpaces.map((space) => (
          <div
            key={space.id}
            role="button"
            tabIndex={0}
            onClick={() => openChatSpaceTab(openTab, space)}
            onKeyDown={(e) => {
              if (e.key === "Enter" || e.key === " ") openChatSpaceTab(openTab, space);
            }}
            className="group flex cursor-pointer items-center gap-2 rounded-md px-2 py-1.5 text-navy/80 hover:bg-teal/25"
          >
            <MessageSquare className="h-3.5 w-3.5 shrink-0 text-navy/40" />
            <span className="flex-1 truncate text-xs">{space.title}</span>
            <button
              type="button"
              aria-label={`Delete chat "${space.title}"`}
              onClick={(e) => handleDeleteChat(e, space)}
              className="shrink-0 rounded-md p-1 text-navy/40 opacity-0 hover:bg-teal/40 hover:text-navy group-hover:opacity-100"
            >
              <Trash2 className="h-3 w-3" />
            </button>
          </div>
        ))}
      </div>
    </div>
  );
}

export default Chatbot;
