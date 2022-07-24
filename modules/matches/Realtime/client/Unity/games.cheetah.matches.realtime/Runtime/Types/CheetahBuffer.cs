using System;
using System.Runtime.InteropServices;
using System.Text;
using Cheetah.Matches.Relay.Codec.Formatter;
using Cheetah.Matches.Relay.Internal.FFI;
using UnityEngine;

namespace Cheetah.Matches.Relay.Types
{
    [StructLayout(LayoutKind.Sequential)]
    public unsafe struct CheetahBuffer
    {
        public byte size;
        public byte pos;

        [MarshalAs(UnmanagedType.ByValArray, SizeConst = Const.MaxSizeStruct)]
        public fixed byte values[Const.MaxSizeStruct];


        public CheetahBuffer(byte[] source) : this()
        {
            foreach (var b in source)
            {
                Add(b);
            }
        }

        private CheetahBuffer Add(byte value)
        {
            values[size] = value;
            size++;
            return this;
        }

        public CheetahBuffer Add(byte[] value)
        {
            foreach (var b in value)
            {
                values[size] = b;
                size++;
            }

            return this;
        }


        public override string ToString()
        {
            var builder = new StringBuilder();
            builder.Append($"Bytes[size = {size},pos = {pos}, data=(");
            for (var i = 0; i < size; i++)
            {
                if (i > 0)
                {
                    builder.Append(" ");
                }

                builder.Append(values[i].ToString("X2"));
            }

            builder.Append(")]");

            return builder.ToString();
        }

        public void Clear()
        {
            size = 0;
        }

        public void AssertEnoughData(uint readSize)
        {
            if (pos + readSize > size)
            {
                Debug.LogError(pos + " " + readSize + " " + size);
                throw new EndOfBufferException();
            }
        }

        public void AssertFreeSpace(uint space)
        {
            if (size + space > Const.MaxSizeStruct)
            {
                throw new IndexOutOfRangeException();
            }
        }
        
        
        internal class EndOfBufferException : Exception
        {
        }
    }
}