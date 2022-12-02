using System;
using System.Collections.Generic;
using Cheetah.Matches.Realtime.Codec;
using Cheetah.Matches.Realtime.Internal;
using Cheetah.Matches.Realtime.Internal.FFI;
using Cheetah.Matches.Realtime.Internal.Plugin;
using Cheetah.Matches.Realtime.Logger;
using Cheetah.Matches.Realtime.Types;
using UnityEngine;

namespace Cheetah.Matches.Realtime
{
    /// <summary>
    /// Клиент Relay сервера
    /// </summary>
    public class CheetahClient
    {
        public readonly CodecRegistry CodecRegistry;
        internal readonly ushort Id;
        public event Action BeforeUpdateHook;
        private readonly Dictionary<Type, object> plugins = new Dictionary<Type, object>();
        private readonly CheetahObjectsCreateInfo objectsCreateInfo;
        private bool enableClientLog = true;
        private ChannelType currentChannelType;
        private byte currentChannelGroup;

        static CheetahClient()
        {
            LoggerGateway.Init();
        }


        public CheetahClient(string address, uint memberId, ulong roomId, byte[] privateUserKey, CodecRegistry codecRegistry)
        {
            LoggerGateway.CollectLogs(false); // очищаем логи с предыдущего клиента
            CodecRegistry = codecRegistry;
            var userPrivateKey = new CheetahBuffer(privateUserKey);
            ResultChecker.Check(ClientFFI.CreateClient(address, (ushort)memberId, roomId, ref userPrivateKey, 0, out Id));
            objectsCreateInfo = GetPlugin<CheetahObjectsCreateInfo>();
        }

        public CheetahClient(string host, uint port, uint memberId, ulong roomId, byte[] privateUserKey, CodecRegistry codecRegistry)
            : this($"{host}:{port}", memberId, roomId, privateUserKey, codecRegistry)
        {
        }

        /// <summary>
        /// Отключить клиентские логи
        /// </summary>
        public void DisableClientLog()
        {
            enableClientLog = false;
        }


        /// <summary>
        /// Обновление состояние. Получение сетевых команд.
        /// </summary>
        public void Update()
        {
            BeforeUpdateHook?.Invoke();
            ResultChecker.Check(ClientFFI.Receive(Id));
            LoggerGateway.CollectLogs(enableClientLog);
        }

        public CheetahClientConnectionStatus GetConnectionStatus()
        {
            ResultChecker.Check(ClientFFI.GetConnectionStatus(Id, out var connectionStatus));
            return connectionStatus;
        }

        public CheetahClientStatistics GetStatistics()
        {
            ResultChecker.Check(ClientFFI.GetStatistics(Id, out var statistics));
            return statistics;
        }

        public T GetPlugin<T>() where T : Plugin, new()
        {
            if (plugins.TryGetValue(typeof(T), out var plugin))
            {
                return (T)plugin;
            }

            var newPlugin = new T();
            newPlugin.Init(this);
            plugins.Add(typeof(T), newPlugin);
            return newPlugin;
        }


        /// <summary>
        /// Создать объект, принадлежащий пользователю
        /// </summary>
        public CheetahObjectBuilder NewObjectBuilder(ushort template, ulong accessGroup)
        {
            return new CheetahObjectBuilder(template, accessGroup, objectsCreateInfo, this);
        }

        public void OnException(Exception e)
        {
            Debug.LogException(e);
        }

        /// <summary>
        /// Зайти в комнату, после этой команды сервер начнет загрузку игровых объектов
        /// </summary>
        public void AttachToRoom()
        {
            ResultChecker.Check(ClientFFI.AttachToRoom(Id));
        }

        /// <summary>
        /// Выйти из комнаты, после этого сервер не будет посылать команды на текущий клиент
        /// </summary>
        public void DetachFromRoom()
        {
            ResultChecker.Check(ClientFFI.DetachFromRoom(Id));
        }

        /// <summary>
        /// Отсоединиться от сервера и удалить информацию о текущем клиенте, после этого методами RelayClient пользоваться нельзя
        /// </summary>
        public void Destroy()
        {
            ResultChecker.Check(ClientFFI.DestroyClient(Id));
        }

        /// <summary>
        /// Получить серверное время (монотонно возрастающее, отсчет от времени запуска сервера)
        /// </summary>
        /// <returns></returns>
        /// <exception cref="ServerTimeNotDefinedException"></exception>
        public ulong GetServerTimeInMs()
        {
            ResultChecker.Check(ClientFFI.GetServerTime(Id, out var time));
            if (time == 0)
            {
                throw new ServerTimeNotDefinedException();
            }

            return time;
        }


        /// <summary>
        /// Установить канал отправки все последующих команд
        /// </summary>
        /// <param name="channelType">тип канала</param>
        /// <param name="group">группа, для групповых каналов, для остальных игнорируется</param>
        public void SetChannelType(ChannelType channelType, byte group)
        {
            if (currentChannelType == channelType && currentChannelGroup == group)
            {
                return;
            }

            currentChannelType = channelType;
            currentChannelGroup = group;
            ResultChecker.Check(ClientFFI.SetChannelType(Id, channelType, group));
        }


        /// <summary>
        /// Сброс эмуляции параметров сети
        /// </summary>
        public void ResetEmulation()
        {
            ResultChecker.Check(ClientFFI.ResetEmulation(Id));
        }

        /// <summary>
        /// Задать параметры эмуляции RTT
        /// Подробнее смотрите в документации проекта
        /// </summary>
        public void SetRttEmulation(ulong rttInMs, double rttDispersion)
        {
            ResultChecker.Check(ClientFFI.SetRttEmulation(Id, rttInMs, rttDispersion));
        }

        /// <summary>
        /// Задать параметры эмуляции потери пакетов
        /// Подробнее смотрите в документации проекта
        /// </summary>
        public void SetDropEmulation(double dropProbability, ulong dropTimeInMs)
        {
            ResultChecker.Check(ClientFFI.SetDropEmulation(Id, dropProbability, dropTimeInMs));
        }
    }

    public class ServerTimeNotDefinedException : Exception
    {
    }
}