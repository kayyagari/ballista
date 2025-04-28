"use client";
import { toast } from "sonner";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import * as z from "zod";
import { cn } from "@/lib/utils";
import { Button } from "@/components/ui/button";
import {
  Form,
  FormControl,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from "@/components/ui/form";
import { Input } from "@/components/ui/input";
import { PasswordInput } from "@/components/ui/password-input";
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "@/components/ui/command";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";
import { Check, ChevronsUpDown, SaveIcon, Trash } from "lucide-react";
import { Textarea } from "@/components/ui/textarea";
import { useConnectionStore } from "@/connections";
import { useParams } from "react-router";
import { useEffect } from "react";

// Add this import for the store state type

const formSchema = z.object({
  connectionName: z.string().min(1),
  connectionURL: z.string().url(),
  username: z.string().min(1),
  password: z.string(),
  path: z.string().min(1),
  group: z.string(),
  note: z.string(),
});

export default function ConnectionForm() {
  const { slug } = useParams();
  const { connections, groups } = useConnectionStore();
  // the cleanname is basicalls url friendly version of the connection name
  const cleanName = (name: string) => {
    let cleanedName = name.toLowerCase().replace(/\s+/g, "-");
    cleanedName = cleanedName.replace(/[^a-z0-9-]/g, "");
    return cleanedName;
  };
  const existingConnection = connections.find(
    (connection) => cleanName(connection.connectionName) === slug
  );
  const form = useForm<z.infer<typeof formSchema>>({
    resolver: zodResolver(formSchema),
    defaultValues: existingConnection
      ? {
          connectionName: existingConnection.connectionName,
          connectionURL: existingConnection.connectionURL,
          username: existingConnection.username,
          password: existingConnection.password,
          path: existingConnection.path,
          group: existingConnection.group,
          note: existingConnection.note,
        }
      : {
          connectionName: "",
          connectionURL: "",
          username: "",
          password: "",
          path: "",
          group: "",
          note: "",
        },
  });

  useEffect(() => {
    form.reset(
      existingConnection
        ? {
            connectionName: existingConnection.connectionName,
            connectionURL: existingConnection.connectionURL,
            username: existingConnection.username,
            password: existingConnection.password,
            path: existingConnection.path,
            group: existingConnection.group,
            note: existingConnection.note,
          }
        : {
            connectionName: "",
            connectionURL: "",
            username: "",
            password: "",
            path: "",
            group: "",
            note: "",
          }
    );
  }, [existingConnection, form, slug]);
  function onDeleteConnection(e: React.MouseEvent<HTMLButtonElement>) {
    e.preventDefault();
    if (existingConnection) {
      useConnectionStore.getState().removeConnection(existingConnection);
      toast.success("Connection deleted successfully!");
    }
  }
  function onSubmit(values: z.infer<typeof formSchema>) {
    try {
      console.log(form.formState.errors);
      const newConnection = {
        id: Math.random().toString(36).substring(2, 15),
        connectionName: values.connectionName,
        connectionURL: values.connectionURL,
        username: values.username,
        password: values.password,
        path: values.path,
        group: values.group,
        note: values.note,
      };
      useConnectionStore.getState().addConnection(newConnection);
      toast("Connection added successfully!", {
        action: {
          label: "go to connection",
          onClick: () => {
            window.location.href = `/${cleanName(
              values.connectionName
            )}`;
          },
        },
      });
      form.reset();
    } catch (error) {
      console.error("Form submission error", error);
      toast.error("Failed to submit the form. Please try again.");
    }
  }

  return (
    <Form {...form}>
      <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4">
        <FormField
          control={form.control}
          name="connectionName"
          render={({ field }) => (
            <FormItem>
              <FormLabel>Connection name </FormLabel>
              <FormControl>
                <Input placeholder="Acme Test Instance" type="" {...field} />
              </FormControl>

              <FormMessage />
            </FormItem>
          )}
        />

        <FormField
          control={form.control}
          name="connectionURL"
          render={({ field }) => (
            <FormItem>
              <FormLabel>MirthConnect URL</FormLabel>
              <FormControl>
                <Input
                  placeholder="https://localhost:8443"
                  type=""
                  {...field}
                />
              </FormControl>

              <FormMessage />
            </FormItem>
          )}
        />

        <FormField
          control={form.control}
          name="username"
          render={({ field }) => (
            <FormItem>
              <FormLabel>Username</FormLabel>
              <FormControl>
                <Input placeholder="e.g admin" type="" {...field} />
              </FormControl>

              <FormMessage />
            </FormItem>
          )}
        />

        <FormField
          control={form.control}
          name="password"
          render={({ field }) => (
            <FormItem>
              <FormLabel>Password</FormLabel>
              <FormControl>
                <PasswordInput placeholder="Skip, if sensitive" {...field} />
              </FormControl>

              <FormMessage />
            </FormItem>
          )}
        />

        <FormField
          control={form.control}
          name="path"
          render={({ field }) => (
            <FormItem>
              <FormLabel>Path to Java Home Directory</FormLabel>
              <FormControl>
                <Input
                  placeholder="C:\Program Files\Zulu\zulu-8\ (For Example)"
                  type=""
                  {...field}
                />
              </FormControl>

              <FormMessage />
            </FormItem>
          )}
        />
        <FormField
          control={form.control}
          name="group"
          render={({ field }) => (
            <FormItem className="flex flex-col">
              <FormLabel>Language</FormLabel>
              <Popover>
                <PopoverTrigger asChild>
                  <FormControl>
                    <Button
                      variant="outline"
                      role="combobox"
                      className={cn(
                        "w-[200px] justify-between",
                        !field.value && "text-muted-foreground"
                      )}
                    >
                      {field.value
                        ? groups.find((group) => group === field.value)
                        : "Select group"}
                      <ChevronsUpDown className="opacity-50" />
                    </Button>
                  </FormControl>
                </PopoverTrigger>
                <PopoverContent className="w-[200px] p-0">
                  <Command>
                    <CommandInput
                      placeholder="Search framework..."
                      className="h-9"
                    />
                    <CommandList>
                      <CommandEmpty>No framework found.</CommandEmpty>
                      <CommandGroup>
                        {groups.map((group) => (
                          <CommandItem
                            value={group}
                            key={group}
                            onSelect={() => {
                              form.setValue("group", group);
                            }}
                          >
                            {group}
                            <Check
                              className={cn(
                                "ml-auto",
                                group === field.value
                                  ? "opacity-100"
                                  : "opacity-0"
                              )}
                            />
                          </CommandItem>
                        ))}
                      </CommandGroup>
                    </CommandList>
                  </Command>
                </PopoverContent>
              </Popover>
              <FormMessage />
            </FormItem>
          )}
        />

        <FormField
          control={form.control}
          name="note"
          render={({ field }) => (
            <FormItem>
              <FormLabel>Note</FormLabel>
              <FormControl>
                <Textarea
                  placeholder="Add a note to this connection"
                  className="resize-none"
                  {...field}
                />
              </FormControl>

              <FormMessage />
            </FormItem>
          )}
        />
        <div className="flex gap-4 w-full justify-center">
          <Button type="submit" className="px-12">
            <SaveIcon />
            Save Connection
          </Button>
          {existingConnection && (
            <Button
              variant={"destructive"}
              onClick={onDeleteConnection}
              className="bg-red-500 px-12"
            >
              <Trash /> Delete Connection
            </Button>
          )}
        </div>
      </form>
    </Form>
  );
}
