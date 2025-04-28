import { AppSidebar } from "@/components/app-sidebar";
import { Separator } from "@/components/ui/separator";
import {
  SidebarInset,
  SidebarProvider,
  SidebarTrigger,
} from "@/components/ui/sidebar";
import { Link, Outlet } from "react-router";
import { Toaster } from "sonner";
import { Button } from "./ui/button";
import { PlusCircleIcon } from "lucide-react";
const Dashbaord = () => {
  return (
    <SidebarProvider>
      <Toaster richColors={true} />
      <AppSidebar />
      <SidebarInset>
        <header className="flex h-16 shrink-0 items-center gap-2">
          <div className="flex items-center gap-2 px-4">
            <SidebarTrigger className="-ml-1" />
            <Separator orientation="vertical" className="mr-2 h-4" />
          </div>
          <div className="flex-1" />
          <div className="flex items-center gap-2 px-4">
            <Link to={"/"}>
              <Button variant={"default"}>
                <span className="material-symbols-outlined">
                  New Connection
                </span>
                <PlusCircleIcon />
              </Button>
            </Link>
          </div>
        </header>
        <div className="flex flex-1 flex-col gap-4 p-4 pt-0">
          <div className="w-full flex justify-center items-center h-full">
            {/* <div className="w-1/2 h-full hidden lg:block px-8">
              <div className=" flex items-end relative overflow-hidden rounded-2xl  w-full h-full ">
                <div
                  className="w-full h-full absolute"
                  style={{
                    backgroundColor: "rgb(8, 145, 178)",
                    background:
                      "radial-gradient(at 31% 13%, rgb(21, 94, 117) 0, transparent 47%), radial-gradient(at 54% 84%, rgb(21, 94, 117) 0, transparent 60%), radial-gradient(at 0% 100%, rgb(12, 74, 110) 0, transparent 51%), radial-gradient(at 82% 22%, rgb(56, 189, 248) 0, transparent 91%), radial-gradient(at 0% 19%, rgb(2, 132, 199) 0, transparent 61%), radial-gradient(at 34% 30%, rgb(28, 25, 23) 0, transparent 67%)",
                  }}
                ></div>
                <div className=" flex z-10 p-8 justify-center items-end ">
                  <img
                    src="/icon.png"
                    alt="Ballista Logo"
                    className=" size-12 mr-2 object-cover"
                  />
                  <h1 className="text-4xl text-white font-bold">Ballista</h1>
                </div>
              </div>
            </div> */}
            <div className="w-1/2">
              <Outlet />
            </div>
          </div>
        </div>
      </SidebarInset>
    </SidebarProvider>
  );
};

export default Dashbaord;
