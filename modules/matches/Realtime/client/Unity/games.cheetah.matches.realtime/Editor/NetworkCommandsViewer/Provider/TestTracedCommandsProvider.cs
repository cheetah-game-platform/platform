using System;
using System.Collections.Generic;
using System.Threading.Tasks;
using Cheetah.Matches.Realtime.GRPC.Admin;

namespace Cheetah.Matches.Realtime.Editor.NetworkCommandsViewer.Provider
{
#pragma warning disable 1998

    public class TestTracedCommandsProvider : TracedCommandsProvider
    {
        private static readonly Random rnd = new Random();
        private readonly List<Command> commands = new List<Command>();

        public override List<Command> GetCommands()
        {
            return commands;
        }


        public override async Task SetRoom(ulong room)
        {
            commands.Clear();
        }

        public override async Task SetFilter(string filter)
        {
            commands.Clear();
        }


        public override async Task<bool> Update()
        {
            commands.Add(GenerateRandomCommand());
            return true;
        }

        public override bool IsReady()
        {
            return true;
        }

        public override void ResetRooms()
        {
        }


        private Command GenerateRandomCommand()
        {
            var command = new Command()
            {
                Direction = rnd.NextDouble() > 0.5 ? "s2c" : "c2s",
                UserId = (uint)rnd.Next(),
                FieldId = (ushort)rnd.Next(),
                Command_ = GetRandom(Commands),
                Template = (uint)GetRandom(Templates),
                Value = GetRandom(Values),
                ObjectId = GetRandom(Objects)
            };
            return command;
        }

        private string GetRandom(string[] values)
        {
            var pos = (uint)(rnd.NextDouble() * values.Length);
            return values[pos];
        }

        private int GetRandom(int[] values)
        {
            var pos = (uint)(rnd.NextDouble() * values.Length);
            return values[pos];
        }


        public override async Task Destroy()
        {
        }

        private static readonly int[] Fields = new int[] { 1, 2, 3, 4, 5 };
        private static readonly int[] Templates = new int[] { 100, 200, 300, 400, 500 };
        private static readonly string[] Commands = new[] { "SetStructure(10)", "IncrementFloatCounter", "DeleteObject", "SetLongValue" };
        private static readonly string[] Objects = new[] { "User(10,10000)", "Root(155)", "Root(255)", "User(50,125)" };
        private static readonly string[] Values = new[] { "{\"alex\"}", " ", "100500", "300" };
    }
#pragma warning restore 1998
}