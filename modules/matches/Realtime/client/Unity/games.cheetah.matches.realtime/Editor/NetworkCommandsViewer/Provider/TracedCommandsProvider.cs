using System;
using System.Collections.Generic;
using System.Threading.Tasks;
using Cheetah.Matches.Realtime.GRPC.Admin;

namespace Cheetah.Matches.Realtime.Editor.NetworkCommandsViewer.Provider
{
#pragma warning disable 1998
    public abstract class TracedCommandsProvider
    {
        public virtual Task SetRoom(ulong room)
        {
            throw new NotImplementedException();
        }

        public virtual async Task SetFilter(string filter)
        {
            throw new NotImplementedException();
        }

        public virtual List<Command> GetCommands()
        {
            throw new NotImplementedException();
        }

        public virtual async Task<bool> Update()
        {
            throw new NotImplementedException();
        }

        /// <summary>
        /// Готов ли провайдер одавать данные
        /// </summary>
        /// <returns></returns>
        public abstract bool IsReady();

        public abstract void ResetRooms();

        public virtual async Task Destroy()
        {
            throw new NotImplementedException();
        }
    }
#pragma warning restore 1998
}