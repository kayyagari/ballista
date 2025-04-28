"use client";

import { useState } from "react";
import { MinusSquareIcon, type LucideIcon } from "lucide-react";

import {
  Collapsible,
  CollapsibleContent,
  CollapsibleTrigger,
} from "@/components/ui/collapsible";
import {
  SidebarGroup,
  SidebarGroupLabel,
  SidebarMenu,
  SidebarMenuAction,
  SidebarMenuButton,
  SidebarMenuItem,
  SidebarMenuSub,
  SidebarMenuSubButton,
  SidebarMenuSubItem,
} from "@/components/ui/sidebar";
import { Link } from "react-router";

export function NavMain({
  items,
}: {
  items: {
    title: string;
    icon: LucideIcon;
    isActive?: boolean;
    items?: {
      title: string;
      url: string;
    }[];
  }[];
}) {
  const [openIndex, setOpenIndex] = useState<number | null>(0);

  return (
    <SidebarGroup>
      <SidebarGroupLabel>Groups</SidebarGroupLabel>
      <SidebarMenu>
        {items.map((item, idx) => (
          <Collapsible
            key={item.title}
            asChild
            open={openIndex === idx}
            onOpenChange={(open) => setOpenIndex(open ? idx : null)}
          >
            <SidebarMenuItem>
              <SidebarMenuButton asChild tooltip={item.title}>
                <CollapsibleTrigger asChild>
                  <div>
                    <SidebarMenuAction>
                      {openIndex === idx ? (
                        // Render a different icon or a modified icon when open
                        <MinusSquareIcon />
                      ) : (
                        <item.icon />
                      )}
                      <span className="sr-only">Toggle</span>
                    </SidebarMenuAction>
                    <span>{item.title}</span>
                  </div>
                </CollapsibleTrigger>
              </SidebarMenuButton>
              {item.items?.length ? (
                <>
                  <CollapsibleContent>
                    <SidebarMenuSub>
                      {item.items?.map((subItem) => (
                        <SidebarMenuSubItem key={subItem.title}>
                          <SidebarMenuSubButton asChild>
                            <Link to={subItem.url}>
                              <span>{subItem.title}</span>
                            </Link>
                          </SidebarMenuSubButton>
                        </SidebarMenuSubItem>
                      ))}
                    </SidebarMenuSub>
                  </CollapsibleContent>
                </>
              ) : null}
            </SidebarMenuItem>
          </Collapsible>
        ))}
      </SidebarMenu>
    </SidebarGroup>
  );
}
