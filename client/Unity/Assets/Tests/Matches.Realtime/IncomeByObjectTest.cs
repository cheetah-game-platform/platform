using System.Threading;
using Games.Cheetah.Client.DOA.Income.ByObject;
using Games.Cheetah.Client.Types;
using NUnit.Framework;
using Shared;
using Shared.Types;
using Tests.Matches.Realtime.Helpers;

namespace Tests.Matches.Realtime
{
    public class IncomeByObjectTest : AbstractTest
    {
        [Test]
        public void TestEventIncomeCommands()
        {
            // создаем объект на первом клиенте
            var createdObject = clientA.NewObjectBuilder(1, PlayerHelper.PlayerGroup).Build();
            // слушаем события определенного типа
            var collector = new EventIncomeByObjectCommandCollector<DropMineEvent>(clientB, createdObject.ObjectId, DropMineEventId);
            // отправляем сообщение
            var dropMineEvent = new DropMineEvent()
            {
                MineId = 150
            };
            clientA.Writer.SendEvent(in createdObject.ObjectId, DropMineEventId, in dropMineEvent);
            // ждем отправки команды
            Thread.Sleep(200);
            // прием команды
            clientB.Update();
            // проверяем результат
            var eventsStream = collector.GetStream();
            var actual = eventsStream.GetItem(0);
            Assert.AreEqual(dropMineEvent.MineId, actual.value.MineId);
            Assert.AreEqual(memberA.UserId, actual.commandCreator);
        }

        [Test]
        public void TestTargetEventIncomeCommands()
        {
            // создаем объект на первом клиенте
            var createdObject = clientA.NewObjectBuilder(1, PlayerHelper.PlayerGroup).Build();
            // слушаем события определенного типа
            var collector = new EventIncomeByObjectCommandCollector<DropMineEvent>(clientB, createdObject.ObjectId, DropMineEventId);
            // отправляем сообщение
            var dropMineEvent = new DropMineEvent()
            {
                MineId = 150
            };
            clientA.Writer.SendEvent(in createdObject.ObjectId, DropMineEventId, memberB.UserId, in dropMineEvent);
            // ждем отправки команды
            Thread.Sleep(200);
            // прием команды
            clientB.Update();
            // проверяем результат
            var eventsStream = collector.GetStream();
            var actual = eventsStream.GetItem(0);
            Assert.AreEqual(dropMineEvent.MineId, actual.value.MineId);
            Assert.AreEqual(memberA.UserId, actual.commandCreator);
        }

        [Test]
        public void TestStructureIncomeCommands()
        {
            // создаем объект на первом клиенте
            var createdObject = clientA.NewObjectBuilder(1, PlayerHelper.PlayerGroup).Build();
            // слушаем события определенного типа
            var collector =
                new StructureIncomeByObjectCommandCollector<TurretsParamsStructure>(clientB, createdObject.ObjectId, TurretsParamsFieldId);
            // изменяем структуру
            var turretsParams = new TurretsParamsStructure()
            {
                Damage = 1.5,
                Speed = 154
            };
            clientA.Writer.SetStructure(in createdObject.ObjectId, TurretsParamsFieldId, in turretsParams);
            // ждем отправки команды
            Thread.Sleep(200);
            // прием команды
            clientB.Update();
            // проверяем результат
            var structuresStream = collector.GetStream();
            var actual = structuresStream.GetItem(0);
            var turretsParamsStructure = actual.value;
            Assert.AreEqual(turretsParams.Damage, turretsParamsStructure.Damage);
            Assert.AreEqual(turretsParams.Speed, turretsParamsStructure.Speed);
            Assert.AreEqual(memberA.UserId, actual.commandCreator);
        }

        [Test]
        public void TestLongIncomeCommands()
        {
            // создаем объект на первом клиенте
            var createdObject = clientA.NewObjectBuilder(1, PlayerHelper.PlayerGroup).Build();
            // слушаем события определенного типа
            var collector = new LongIncomeByObjectCommandCollector(clientB, createdObject.ObjectId, ScoreFieldId);
            // изменяем значение
            clientA.Writer.SetLong(in createdObject.ObjectId, ScoreFieldId, 7799);
            // ждем отправки команды
            Thread.Sleep(200);
            // прием команды
            clientB.Update();
            // проверяем результат
            var stream = collector.GetStream();
            var actual = stream.GetItem(0);
            Assert.AreEqual(7799, actual.value);
            Assert.AreEqual(memberA.UserId, actual.commandCreator);
        }


        [Test]
        public void TestDoubleIncomeCommands()
        {
            // создаем объект на первом клиенте
            var createdObject = clientA.NewObjectBuilder(1, PlayerHelper.PlayerGroup).Build();
            // слушаем события определенного типа
            var collector = new DoubleIncomeByObjectCommandCollector(clientB, createdObject.ObjectId, HealFieldId);
            // изменяем значение
            clientA.Writer.SetDouble(in createdObject.ObjectId, HealFieldId, 77.99);
            // ждем отправки команды
            Thread.Sleep(200);
            // прием команды
            clientB.Update();
            // проверяем результат
            var stream = collector.GetStream();
            var actual = stream.GetItem(0);
            Assert.AreEqual(77.99, actual.value);
            Assert.AreEqual(memberA.UserId, actual.commandCreator);
        }

        [Test]
        public void TestDeleteFieldCommands()
        {
            // создаем объект на первом клиенте
            var createdObject = clientA.NewObjectBuilder(1, PlayerHelper.PlayerGroup).Build();
            // слушаем события определенного типа
            var collector = new DeleteFieldIncomeByObjectCommandCollector(clientB, createdObject.ObjectId, ScoreFieldId);
            clientA.Writer.SetLong(in createdObject.ObjectId, ScoreFieldId, 100);
            // удаляем поле
            clientA.Writer.DeleteField(in createdObject.ObjectId, ScoreFieldId);

            // ждем отправки команды
            Thread.Sleep(200);
            // прием команды
            clientB.Update();
            // проверяем результат
            var stream = collector.GetStream();
            var actual = stream.GetItem(0);
            Assert.AreEqual(FieldType.Long, actual.value);
            Assert.AreEqual(memberA.UserId, actual.commandCreator);
        }
    }
}