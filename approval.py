DANGEROUS_COMMANDS = [
    "rm -rf",
    "shutdown",
    "reboot",
    "mkfs",
    "dd ",
]


def requires_approval(tool_name, arguments):
    if tool_name != "shell":
        return False
    command = arguments.get("command", "")
    return any(dangerous in command for dangerous in DANGEROUS_COMMANDS)


def approve(tool_name, arguments):
    if not requires_approval(tool_name, arguments):
        return True
    print()
    print("APPROVAL REQUIRED")
    print(f"Tool: {tool_name}")
    print(f"Arguments: {arguments}")
    answer = input("Allow? [y/N] ")
    return answer.lower() == "y"
