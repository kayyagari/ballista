import * as React from "react";
import { PlusCircleIcon, PlusSquareIcon } from "lucide-react";

import { NavMain } from "@/components/nav-main";
import {
  Sidebar,
  SidebarContent,
  SidebarFooter,
  SidebarHeader,
  SidebarMenu,
  SidebarMenuButton,
  SidebarMenuItem,
} from "@/components/ui/sidebar";
import { Input } from "./ui/input";
import { Button } from "./ui/button";
import { useConnectionStore } from "@/connections";
import { Popover, PopoverContent, PopoverTrigger } from "./ui/popover";
import { useDebounce } from "@/hooks/use-debounce";

export function AppSidebar({ ...props }: React.ComponentProps<typeof Sidebar>) {
  const { groups, addGroup } = useConnectionStore();
  const connections = useConnectionStore((state) => state.connections);
  const [newGroup, setNewGroup] = React.useState("");
  const [search, setSearch] = React.useState("");
  const debouncedSearch = useDebounce(search, 300); // useDebounce for search

  const handleAddGroup = () => {
    addGroup(newGroup);
    setNewGroup("");
  };

  // Filter connections by debounced search query
  const filteredConnections = connections.filter((conn) =>
    conn.connectionName.toLowerCase().includes(debouncedSearch.toLowerCase())
  );

  // Only include groups that have at least one filtered connection
  const groupsWithConnections = groups
    .map((group) => {
      const groupConnections = filteredConnections.filter((conn) => conn.group === group);
      return {
        group: group,
        connections: groupConnections,
      };
    })
    .filter((groupObj) => groupObj.connections.length > 0); // filter out empty groups

  const data = {
    navMain: groupsWithConnections.map((groupObj) => ({
      title: groupObj.group,
      icon: PlusSquareIcon,
      isActive: true,
      items: groupObj.connections.map((conn) => ({
        title: conn.connectionName,
        url: `/${conn.connectionName.replace(/\s+/g, "_").toLowerCase()}`,
      })),
    })),
  };

  return (
    <Sidebar variant="inset" {...props}>
      <SidebarHeader>
        <SidebarMenu>
          <SidebarMenuItem>
            <SidebarMenuButton size="default" asChild>
              <Input
                placeholder="search..."
                value={search}
                onChange={(e) => setSearch(e.target.value)}
              />
            </SidebarMenuButton>
          </SidebarMenuItem>
        </SidebarMenu>
      </SidebarHeader>
      <SidebarContent>
        <NavMain items={data.navMain} />
      </SidebarContent>
      <SidebarFooter>
        <div className="w-full">
          <Popover>
            <PopoverTrigger className="w-full" asChild>
              <Button className="w-full" variant="outline">
                Add New Group
              </Button>
            </PopoverTrigger>
            <PopoverContent>
              <div className="flex gap-4">
                <Input
                  placeholder="group name e.g production "
                  value={newGroup}
                  onChange={(e) => setNewGroup(e.target.value)}
                  onKeyDown={(e) => {
                    if (e.key === "Enter") {
                      handleAddGroup();
                    }
                  }}
                />
                <Button
                  className="cursor-pointer"
                  onClick={handleAddGroup}
                >
                  <PlusCircleIcon />
                </Button>
              </div>
            </PopoverContent>
          </Popover>
        </div>
      </SidebarFooter>
    </Sidebar>
  );
}
