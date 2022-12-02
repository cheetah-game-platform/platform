using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using Cheetah.Matches.Realtime.GRPC.Admin;
using Cheetah.Matches.Realtime.GRPC.Shared;
using Google.Protobuf;

namespace Cheetah.Matches.Realtime.Editor.DumpViewer.Provider
{
#pragma warning disable 1998
    public class TestDumpProvider : DumpProvider
    {
        private readonly Random random = new Random();
        private const int UserCount = 200;
        private const int RoomObjectsCount = 1000;

        public async Task<DumpResponse> Dump(ulong room)
        {
            var response = new DumpResponse();
            GenerateObjects(response, 0, true, RoomObjectsCount);
            GenerateUsersObjects(response);
            return response;
        }

        public async Task Destroy()
        {
        }

        private void GenerateUsersObjects(DumpResponse response)
        {
            var users = new List<DumpUser>();
            for (uint userId = 0; userId < UserCount; userId++)
            {
                var objectsCount = random.Next(10, 50);
                var user = new DumpUser
                {
                    Id = userId,
                    Groups = GenerateGroups(),
                    Attached = random.NextDouble() > 0.3,
                };
                users.Add(user);
                GenerateObjects(response, userId, false, objectsCount);
            }

            var rnd = new Random();
            foreach (var o in users.OrderBy(i => rnd.Next()))
            {
                response.Users.Add(o);
            }
        }

        private void GenerateObjects(DumpResponse response, uint owner, bool roomOwner, int count)
        {
            var objects = new List<DumpObject>();
            for (uint objectId = 0; objectId < count; objectId++)
            {
                var obj = GenerateObject(objectId, owner, roomOwner);
                objects.Add(obj);
            }

            var rnd = new Random();
            foreach (var o in objects.OrderBy(i => rnd.Next()))
            {
                response.Objects.Add(o);
            }
        }

        private DumpObject GenerateObject(uint id, uint owner, bool roomOwner)
        {
            var obj = new DumpObject
            {
                Template = (ushort)random.Next(1, 10),
                Created = random.NextDouble() > 0.2,
                OwnerUserId = owner,
                Id = id,
                Groups = GenerateGroups(),
            };
            if (roomOwner)
            {
                obj.ClearOwnerUserId();
            }

            GenerateDoubles(obj);
            GenerateLongs(obj);
            GenerateStructures(obj);
            return obj;
        }

        private ulong GenerateGroups()
        {
            var groups = new[] { 1, 2, 4, 5, 8, 16 };
            return (ulong)groups[random.Next(0, groups.Length - 1)];
        }

        private void GenerateStructures(DumpObject obj)
        {
            var count = random.Next(10, 50);
            for (uint fieldId = 0; fieldId < count; fieldId++)
            {
                obj.Fields.Add(new GameObjectField()
                {
                    Id = fieldId,
                    Value = new FieldValue
                    {
                        Structure = ByteString.CopyFromUtf8("content " + fieldId)
                    }
                });
            }
        }

        private void GenerateLongs(DumpObject obj)
        {
            var count = random.Next(10, 50);
            for (uint fieldId = 0; fieldId < count; fieldId++)
            {
                obj.Fields.Add(new GameObjectField()
                {
                    Id = fieldId,
                    Value = new FieldValue
                    {
                        Long = random.Next(0, int.MaxValue)
                    }
                });
                if (random.NextDouble() > 0.5)
                {
                    obj.CompareAndSetOwners.Add(fieldId, random.Next(0, UserCount));
                }
            }
        }

        private void GenerateDoubles(DumpObject obj)
        {
            var count = random.Next(10, 50);
            for (uint fieldId = 0; fieldId < count; fieldId++)
            {
                obj.Fields.Add(new GameObjectField()
                {
                    Id = fieldId,
                    Value = new FieldValue
                    {
                        Double = random.NextDouble()
                    }
                });
            }
        }
    }
#pragma warning restore 1998
}