using System.Net;
using System.Threading;
using System.Threading.Tasks;
using Games.Cheetah.Client;
using Games.Cheetah.Client.Codec;
using Games.Cheetah.EmbeddedServer.API;
using Games.Cheetah.GRPC.Internal;
using NUnit.Framework;
using Shared;
using Shared_Types;

namespace Tests.Matches.Realtime.Helpers
{
    public abstract class AbstractTest
    {
        protected CheetahClient clientA;

        protected CheetahClient clientB;
        private RoomIdResponse roomIdResponse;
        protected CreateMemberResponse memberA;
        protected CreateMemberResponse memberB;
        private EmbeddedServer server;
        protected static FieldId.Structure TurretsParamsFieldId = new(100);
        protected static FieldId.Event DropMineEventId = new(555);
        protected static FieldId.Double HealFieldId = new(777);
        protected static FieldId.Long ScoreFieldId = new(999);


        [SetUp]
        public void SetUp()
        {
            server = new EmbeddedServer(IPAddress.Loopback);
            var grpcClient = server.CreateGrpcClient();
            Task.Run(async () =>
            {
                roomIdResponse = await grpcClient.CreateRoomAsync(new RoomTemplate());
                memberA = await grpcClient.CreateMemberAsync(new CreateMemberRequest
                {
                    RoomId = roomIdResponse.RoomId,
                    User = new UserTemplate
                    {
                        Groups = PlayerHelper.PlayerGroup
                    }
                });
                memberB = await grpcClient.CreateMemberAsync(new CreateMemberRequest
                {
                    RoomId = roomIdResponse.RoomId,
                    User = new UserTemplate
                    {
                        Groups = PlayerHelper.PlayerGroup
                    }
                });
            }).GetAwaiter().GetResult();


            var codecRegistry = new CodecRegistryBuilder();
            codecRegistry.Register(_ => new GlobalNamespaceObjectCodec());
            codecRegistry.Register(_ => new DropMineEventCodec());
            codecRegistry.Register(_ => new SomeSingletonKeyCodec());
            codecRegistry.Register(_ => new TurretsParamsStructureCodec());


            // подключаем первого клиента
            clientA = ConnectToServer(server, roomIdResponse.RoomId, memberA, codecRegistry);
            clientA.AttachToRoom();

            // подключаем второго клиента
            clientB = ConnectToServer(server, roomIdResponse.RoomId, memberB, codecRegistry);
            clientB.AttachToRoom();

            // полуаем сетевые команды, которые не надо учитывать в тестах
            Thread.Sleep(200);
            clientA.Update();
            clientB.Update();
        }

        private static CheetahClient ConnectToServer(EmbeddedServer server, ulong roomId, CreateMemberResponse member,
            CodecRegistryBuilder codecRegistryBuilder)
        {
            var client = new CheetahClient(server.GetUdpGameHost(), server.GetUdpGamePort(), member.UserId, roomId,
                member.PrivateKey.ToByteArray(),
                codecRegistryBuilder.Build());
            client.DisableClientLog();
            return client;
        }

        [TearDown]
        public void TearDown()
        {
            clientA.Dispose();
            clientB.Dispose();
            server.Destroy();
        }
    }
}