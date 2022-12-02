using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using Cheetah.Matches.Realtime.GRPC.Admin;
using Cheetah.Platform;
using UnityEditor;

namespace Cheetah.Matches.Realtime.Editor.NetworkCommandsViewer.Provider
{
    /// <summary>
    /// Провайдер команд с реального сервера
    /// </summary>
    public class RemoteTracedCommandsProvider : TracedCommandsProvider
    {
        private ClusterConnector connector;
        private const int CommandCountLimit = 5000;
        private readonly IDictionary<ulong, uint> roomToSession = new Dictionary<ulong, uint>();
        private bool sessionCreated;
        private uint currentSession;
        private ulong currentRoom;
        private string currentFilter;
        private readonly List<Command> commands = new List<Command>();

        public RemoteTracedCommandsProvider(ClusterConnector connector)
        {
            this.connector = connector;
            EditorApplication.quitting += ApplicationQuitting;
        }

        public override async Task SetRoom(ulong room)
        {
            commands.Clear();
            if (!roomToSession.TryGetValue(room, out currentSession))
            {
                var result = await connector.DoRequest(async channel =>
                {
                    var client = new CommandTracer.CommandTracerClient(channel);
                    return await client.CreateSessionAsync(new CreateSessionRequest
                    {
                        Room = room
                    });
                });
                currentSession = result.Id;
                currentRoom = room;
                roomToSession[room] = currentSession;
                sessionCreated = true;
            }

            await SetFilter(currentFilter);
        }

        public override async Task SetFilter(string filter)
        {
            if (filter == null)
            {
                return;
            }

            currentFilter = filter;

            if (!sessionCreated)
            {
                return;
            }


            await connector.DoRequest(async channel =>
            {
                var client = new CommandTracer.CommandTracerClient(channel);
                return await client.SetFilterAsync(new SetFilterRequest
                {
                    Filter = filter,
                    Room = currentRoom,
                    Session = currentSession
                });
            });
            commands.Clear();
        }

        public override List<Command> GetCommands()
        {
            return commands;
        }

        public override async Task<bool> Update()
        {
            if (!sessionCreated)
            {
                return false;
            }

            var result = await connector.DoRequest(async channel =>
            {
                var client = new CommandTracer.CommandTracerClient(channel);
                return await client.GetCommandsAsync(new GetCommandsRequest
                {
                    Room = currentRoom,
                    Session = currentSession
                });
            });
            var containsCommand = commands.Count > 0;
            foreach (var command in result.Commands.ToList())
            {
                commands.Add(command);
            }

            while (commands.Count > CommandCountLimit)
            {
                commands.RemoveAt(0);
            }

            return containsCommand;
        }

        public override bool IsReady()
        {
            return sessionCreated;
        }

        public override void ResetRooms()
        {
            sessionCreated = false;
            currentRoom = 0;
            currentSession = 0;
        }

        public override async Task Destroy()
        {
            var tmpConnector = connector;
            connector = null;
            await tmpConnector.Destroy();
        }

        public async void ApplicationQuitting()
        {
            await Destroy();
        }
    }
}