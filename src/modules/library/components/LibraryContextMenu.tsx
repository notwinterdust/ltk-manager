import { FolderPlus } from "lucide-react";
import { useState } from "react";

import { ContextMenu, Dialog } from "@/components";
import { useCreateFolder } from "@/modules/library/api";

import { FolderNameForm } from "./FolderNameForm";

interface LibraryContextMenuProps {
  children: React.ReactNode;
}

export function LibraryContextMenu({ children }: LibraryContextMenuProps) {
  const [dialogOpen, setDialogOpen] = useState(false);
  const createFolder = useCreateFolder();

  return (
    <>
      <ContextMenu.Root>
        <ContextMenu.Trigger className="flex min-h-0 flex-1 flex-col">
          {children}
        </ContextMenu.Trigger>
        <ContextMenu.Portal>
          <ContextMenu.Positioner>
            <ContextMenu.Popup>
              <ContextMenu.Item
                icon={<FolderPlus className="h-4 w-4" />}
                onClick={() => setDialogOpen(true)}
              >
                New Folder
              </ContextMenu.Item>
            </ContextMenu.Popup>
          </ContextMenu.Positioner>
        </ContextMenu.Portal>
      </ContextMenu.Root>
      <Dialog.Root open={dialogOpen} onOpenChange={setDialogOpen}>
        <Dialog.Portal>
          <Dialog.Backdrop />
          <Dialog.Overlay size="sm">
            <div className="p-5">
              <Dialog.Title>New Folder</Dialog.Title>
              <div className="mt-3">
                <FolderNameForm
                  submitLabel="Create"
                  isPending={createFolder.isPending}
                  onSubmit={(name) =>
                    createFolder.mutate(name, { onSuccess: () => setDialogOpen(false) })
                  }
                  onCancel={() => setDialogOpen(false)}
                />
              </div>
            </div>
          </Dialog.Overlay>
        </Dialog.Portal>
      </Dialog.Root>
    </>
  );
}
